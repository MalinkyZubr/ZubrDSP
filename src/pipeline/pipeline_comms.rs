use tokio::sync::mpsc::{Receiver, Sender, error::{SendError, TryRecvError, TrySendError}, channel};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Notify;
use crate::pipeline::errors::PipelineError;
use super::pipeline_traits::Sharable;


#[derive(Debug, Copy)]
pub enum Message<T: Sharable> {
    Data(T),
    Kill
}
impl<T: Sharable> Message<T> {
    pub fn get(self) -> Option<T> {
        match self {
            Message::Data(value) => Some(value),
            Message::Kill => None
        }
    }
}


pub enum RealSender<T: crate::pipeline::pipeline_traits::Sharable> {
    Real(Sender<Message<T>>),
    Dummy
}


// turn all these to be async
impl<T: crate::pipeline::pipeline_traits::Sharable> RealSender<T> {
    pub async fn send(&mut self, data: T) -> Result<(), SendError<Message<T>>> {
        match self {
            RealSender::Real(sender) => sender.send(Message::Data(data)).await,
            RealSender::Dummy => Result::Ok(())
        }
    }

    pub fn try_send(&mut self, data: T) -> (Result<(), TrySendError<Message<T>>>) {
        match self {
            RealSender::Real(sender) => sender.try_send(Message::Data(data)),
            RealSender::Dummy => Result::Ok(())
        }
    }
}

pub enum RealReceiver<T: crate::pipeline::pipeline_traits::Sharable> {
    Real(Receiver<Message<T>>),
    Dummy
}
impl<T: crate::pipeline::pipeline_traits::Sharable> RealReceiver<T> {
    pub async fn recv(&mut self) -> (Option<Message<T>>, bool) {
        match self {
            RealReceiver::Real(receiver) => (receiver.recv().await, true),
            RealReceiver::Dummy => (None, false)
        }
    }

    // pub fn recv_timeout(&mut self, timeout: core::time::Duration) -> (Result<T, RecvTimeoutError>, bool) {
    //     match self {
    //         RealReceiver::Real(receiver) => (receiver.recv_timeout(timeout), true),
    //         RealReceiver::Dummy => (Result::Err(RecvTimeoutError::Timeout), false)
    //     }
    // }
}


pub fn wrapped_channel<T: Sharable>(buffer_size: usize) -> ((RealSender<Message<T>>, RealSender<Message<T>>), RealReceiver<Message<T>>) {
    let (tx1, rx) = channel(buffer_size);
    let tx2 = tx1.clone(); // jack the (async) ripper

    (
        (RealSender::Real(tx1), RealSender::Real(tx2)),
        RealReceiver::Real(rx)
    )
}


pub struct MultiSender<T: Sharable> {
    senders: Vec<RealSender<T>>
}
impl<T: Sharable> MultiSender<T> {
    pub fn send_all(&mut self, mut data: Vec<T>) {
        data.reverse();
        for index in 0..data.len() {
            let data_point = data.pop().unwrap();
            self.senders[index].try_send(data_point);
        }
    }
}


pub struct MultiReceiver<T: Sharable> {
    receivers: Vec<tokio::task::JoinHandle<()>>,
    done_notifier: Arc<Notify>,
    read_finish_notifier: Arc<Notify>,
    output: Arc<RwLock<Vec<T>>>,
    completion_count: Arc<AtomicUsize>,
    num_receivers: usize
}
impl<T: Sharable> MultiReceiver<T> {
    // pub fn new() -> (MultiReceiver<T>, Notify) {
    //
    // }
    pub fn extract_data(&mut self, data: &mut Vec<T>) { // assume space already allocated for the data vector
        data.copy_from_slice(self.output.read().unwrap().as_slice());
        self.read_finish_notifier.notify_waiters();
    }

    fn spawn_task(&mut self, data_index: usize, mut receiver: RealReceiver<Message<T>>, output_index: usize) {
        let data_buffer = self.output.clone();
        let counter = self.completion_count.clone();
        let necessary_receives = self.num_receivers.clone();
        let mut done_notifier = self.done_notifier.clone();
        let mut read_finish_notifier = self.read_finish_notifier.clone();

        self.receivers.push(
            tokio::spawn(
                async move {
                    let mut runflag = true;

                    while runflag {
                        let message = receiver.recv().await;

                        let mut data = None;

                        match message {
                            (Some(Message::Data(value)), true) => data = value.get(),
                            (Some(Message::Kill), true) => {runflag = false; continue},
                            (_, false) => runflag = panic!("Receiver receive error"), // what do here
                            (None, _) => panic!("Receiver got None message")
                        }

                        {
                            let mut data_buffer = data_buffer.write().unwrap();
                            data_buffer[output_index] = data.unwrap();
                        }

                        counter.fetch_add(1, Ordering::SeqCst);
                        if counter.load(Ordering::SeqCst) == necessary_receives {
                            done_notifier.notify_waiters();
                            counter.store(0, Ordering::SeqCst);
                        }

                        read_finish_notifier.notified().await;
                    }
                }
            )
        )
    }
}


pub enum NodeCommunicator<I: Sharable, O: Sharable> {
    SISO(RealReceiver<I>, RealSender<O>),
    SIMO(RealReceiver<I>, MultiSender<O>),
    MISO(MultiReceiver<I>, RealSender<O>)
}
impl<I: Sharable, O: Sharable> NodeCommunicator<I, O> {
    pub async fn receive(&mut self) -> (Option<I>, PipelineError ) {
        match &mut self {
            NodeCommunicator::SISO(receiver, _) |
            NodeCommunicator::SIMO(receiver, _) => receiver.recv(),
            NodeCommunicator::MISO(receiver, _) => panic!("MISO not implemented")
        }
    }
    
    fn receive_single(&mut self) -> (Option<I>, PipelineError) { // make interruption of the process an async process, dont use timeouts. Wastes compute
        match self.input.recv_timeout(std::time::Duration::from_millis(100)) { // add some iterators here to add multi in-multi out functionality
            (Ok(data), _) => (Some(data), PipelineError::Ok),
            (Err(RecvTimeoutError), true) => {
                //println!("ID: {} failed to receive!", &self.id);
                (None, PipelineError::ReceiveError)
            }
            (Err(RecvTimeoutError), false) => {
                (None, PipelineError::Timeout)
            } // make into separate function
        }
    }

    fn send_single(&mut self, output_data: O) -> PipelineError {
        let return_error = match self.output.send(output_data) {
            Ok(()) => PipelineError::Ok,
            Err(_msg) => PipelineError::SendError
        };

        return return_error;
    }
}