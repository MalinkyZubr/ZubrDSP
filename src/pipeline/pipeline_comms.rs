use tokio::sync::mpsc::{Receiver, Sender, error::{SendError, TryRecvError, TrySendError}, channel};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Notify;
use crate::pipeline::errors::{CommType, PipelineCommResult};
use super::pipeline_traits::Sharable;


#[derive(Debug, Clone, Copy)]
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


pub enum WrappedSender<T: Sharable> {
    Real(Sender<Message<T>>),
    Dummy
}


// turn all these to be async
impl<T: Sharable> WrappedSender<T> {
    pub async fn send(&mut self, data: T) -> PipelineCommResult<()> {
        match self {
            WrappedSender::Real(sender) => {
                let result = sender.send(Message::Data(data)).await;
                match result {
                    Err(SendError(message)) => PipelineCommResult::CommError(CommType::Sender),
                    Ok(()) => PipelineCommResult::Ok(())
                }
            },
            WrappedSender::Dummy => PipelineCommResult::ResourceNotExist
        }
    }
    // pub fn try_send(&mut self, data: T) -> PipelineCommResult<()> {
    //     match self {
    //         WrappedSender::Real(sender) => sender.try_send(Message::Data(data)),
    //         WrappedSender::Dummy => Result::Ok(())
    //     }
    // }
}

pub enum WrappedReceiver<T: Sharable> {
    Real(Receiver<Message<T>>),
    Dummy
}

impl<T: Sharable> WrappedReceiver<T> {
    pub async fn recv(&mut self) -> PipelineCommResult<Message<T>> {
        match self {
            WrappedReceiver::Real(receiver) => {
                let result = receiver.recv().await;
                match result {
                    None => PipelineCommResult::CommError(CommType::Receiver),
                    Some(message) => PipelineCommResult::Ok(message),
                }
            },
            WrappedReceiver::Dummy => PipelineCommResult::ResourceNotExist
        }
    }

    // pub fn recv_timeout(&mut self, timeout: core::time::Duration) -> (Result<T, RecvTimeoutError>, bool) {
    //     match self {
    //         WrappedReceiver::Real(receiver) => (receiver.recv_timeout(timeout), true),
    //         WrappedReceiver::Dummy => (Result::Err(RecvTimeoutError::Timeout), false)
    //     }
    // }
}


pub fn wrapped_channel<T: Sharable>(buffer_size: usize) -> ((WrappedSender<Message<T>>, WrappedSender<Message<T>>), WrappedReceiver<Message<T>>) {
    let (tx1, rx) = channel(buffer_size);
    let tx2 = tx1.clone(); // jack the (async) ripper

    (
        (WrappedSender::Real(tx1), WrappedSender::Real(tx2)),
        WrappedReceiver::Real(rx)
    )
}


pub struct SingleSender<T: Sharable> {
    sender: WrappedSender<T>
}
impl <T: Sharable> SingleSender<T> {
    pub fn send_single(&mut self, output_data: T) -> PipelineError {
        let return_error = match self.sender.try_send(output_data) {
            Ok(()) => PipelineError::Ok,
            Err(_msg) => PipelineError::SendError
        };

        return return_error;
    }
}


pub struct MultiSender<T: Sharable> {
    senders: Vec<WrappedSender<T>>
}
impl<T: Sharable> MultiSender<T> {
    pub async fn send_all(&mut self, mut data: Vec<T>) -> Vec<PipelineCommResult<()>> {
        data.reverse();
        let mut results = Vec::with_capacity(data.len());
        for index in 0..data.len() {
            let data_point = data.pop().unwrap();
            results.push(self.senders[index].send(data_point).await); // evaluate this decision later
        }
        
        return results;
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
    pub fn extract_data(&mut self, mut data: Vec<T>) -> Vec<PipelineCommResult<T>> { // assume space already allocated for the data vector
        data.copy_from_slice(self.output.read().unwrap().as_slice());
        self.read_finish_notifier.notify_waiters();
    }

    fn spawn_task(&mut self, mut receiver: WrappedReceiver<Message<T>>, output_index: usize) {
        let data_buffer = self.output.clone();
        let counter = self.completion_count.clone();
        let necessary_receives = self.num_receivers.clone();
        let done_notifier = self.done_notifier.clone();
        let read_finish_notifier = self.read_finish_notifier.clone();

        self.receivers.push(tokio::spawn(async move {
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
        }))
    }
}


pub enum NodeCommunicator<I: Sharable, O: Sharable> {
    SISO(SingleReceiver<I>, SingleSender<O>),
    SIMO(SingleReceiver<I>, MultiSender<O>),
    MISO(MultiReceiver<I>, SingleSender<O>)
}
impl<I: Sharable, O: Sharable> NodeCommunicator<I, O> {
    pub async fn receive(&mut self) -> Vec<PipelineCommResult<I>> {
        match &mut self {
            NodeCommunicator::SISO(receiver, _) |
            NodeCommunicator::SIMO(receiver, _) => {
                
            },
            NodeCommunicator::MISO(receiver, _) => {
                
            }
        }
    }
}