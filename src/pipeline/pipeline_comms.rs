use tokio::sync::mpsc::{Receiver, Sender, error::{SendError, TryRecvError, TrySendError}, channel};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Notify;
use crate::pipeline::pipeline_results::{CommType, PipelineCommResult};
use super::pipeline_traits::Sharable;


#[derive(Debug, Clone, Copy)]
pub enum ControlPlaneDirective {
    Start,
    Stop,
    Kill
}
#[derive(Debug, Clone, Copy)]
pub enum Message<T: Sharable> {
    Data(T),
    Directive(ControlPlaneDirective)
}
impl<T: Sharable> Message<T> {
    pub fn unwrap(self) -> T {
        match self {
            Message::Data(value) => value,
            Message::Directive(_) => panic!("Cannot unwrap a message")
        }
    }
    pub fn get_data(&mut self) -> &mut T {
        match self {
            Message::Data(t) => t,
            _ => panic!("Could not get Data")
        }
    }
}


impl<T: Sharable> Message<T> {
    pub fn get(self) -> Option<T> {
        match self {
            Message::Data(value) => Some(value),
            Message::Directive(_) => None
        }
    }
}


pub trait MessageHandler<T: Sharable, R> {
    fn handle_message(&mut self, message: Message<T>) -> R;
}

pub trait ControlPlaneHandler<R> {
    fn handle_cp_directive(&mut self, message: ControlPlaneDirective) -> R;
}

pub trait PipelineCommHandler<T: Sharable, R> {
    fn handle_comm_result(&mut self, message: PipelineCommResult<T>) -> R;
}

pub trait StatelessMessageHandler<T: Sharable, R> {
    fn handle_message(message: Message<T>) -> R;
}

pub trait StatelessControlPlaneHandler<R> {
    fn handle_cp_directive(message: ControlPlaneDirective) -> R;
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
}

pub enum WrappedReceiver<T: Sharable, const ACCEPT_CONTROL_PLANE: bool> {
    Real(Receiver<Message<T>>),
    Dummy
}

impl<T: Sharable, const ACCEPT_CONTROL_PLANE: bool> WrappedReceiver<T, ACCEPT_CONTROL_PLANE> {
    pub async fn recv(&mut self) -> PipelineCommResult<Message<T>> {
        match self { 
            WrappedReceiver::Real(receiver) => {
                let result = receiver.recv().await;
                match result {
                    None => PipelineCommResult::CommError(CommType::Receiver),
                    Some(message) => self.handle_message(message)
                }
            },
            WrappedReceiver::Dummy => PipelineCommResult::ResourceNotExist
        }
    }
}
impl<T: Sharable, const ACCEPT_CONTROL_PLANE: bool> MessageHandler<T, PipelineCommResult<Message<T>>> for WrappedReceiver<T, ACCEPT_CONTROL_PLANE> {
    fn handle_message(&mut self, message: Message<T>) -> PipelineCommResult<Message<T>> {
        if let Message::Directive(directive) = message && !ACCEPT_CONTROL_PLANE {
            PipelineCommResult::IllegalDirective
        }
        else {
            PipelineCommResult::Ok(message)
        }
    }
}


pub fn wrapped_channel<T: Sharable, const ACCEPT_CONTROL_PLANE: bool>(buffer_size: usize) -> ((WrappedSender<T>, Sender<Message<T>>), WrappedReceiver<T, ACCEPT_CONTROL_PLANE>) {
    let (tx1, rx) = channel::<Message<T>>(buffer_size);
    let tx2 = tx1.clone(); // jack the (async) ripper

    (
        (WrappedSender::Real(tx1), tx2),
        WrappedReceiver::Real(rx)
    )
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
    output: Arc<RwLock<Vec<PipelineCommResult<Message<T>>>>>,
    completion_count: Arc<AtomicUsize>,
    num_receivers: usize
}
struct MultiReceiverFlags {
    kill_flag: bool,
    run_flag: bool
}

impl<T: Sharable> MultiReceiver<T> {
    // pub fn new() -> (MultiReceiver<T>, Notify) {
    //
    // }
    pub async fn extract_data(&mut self) -> Vec<PipelineCommResult<Message<T>>> { // assume space already allocated for the data vector
        self.done_notifier.notified().await;
        let mut data = Vec::with_capacity(self.output.read().unwrap().len());
        data.copy_from_slice(self.output.read().unwrap().as_slice());
        self.read_finish_notifier.notify_waiters();

        return data;
    }

    fn spawn_task(&mut self, mut receiver: WrappedReceiver<T, true>, output_index: usize) {
        let data_buffer = self.output.clone();
        let counter = self.completion_count.clone();
        let necessary_receives = self.num_receivers.clone();
        let done_notifier = self.done_notifier.clone();
        let read_finish_notifier = self.read_finish_notifier.clone();

        self.receivers.push(tokio::spawn(async move {
            let mut flags = MultiReceiverFlags { kill_flag: false, run_flag: false };

            while !flags.kill_flag {
                flags.handle_comm_result(receiver.recv().await);
                
                while flags.run_flag {
                    let mut message = receiver.recv().await;

                    message = flags.handle_comm_result(message);
                    if flags.kill_flag || !flags.run_flag {continue;}

                    {
                        let mut data_buffer = data_buffer.write().unwrap();
                        data_buffer[output_index] = message;
                    }

                    counter.fetch_add(1, Ordering::SeqCst);
                    if counter.load(Ordering::SeqCst) == necessary_receives {
                        done_notifier.notify_waiters();
                        counter.store(0, Ordering::SeqCst);
                    }

                    read_finish_notifier.notified().await;
                }   
            }
        }))
    }
}
impl<T: Sharable> PipelineCommHandler<Message<T>, PipelineCommResult<Message<T>>> for MultiReceiverFlags {
    fn handle_comm_result(&mut self, message: PipelineCommResult<Message<T>>) -> PipelineCommResult<Message<T>> {
        match message {
            PipelineCommResult::Ok(message_content) => {
                let message_content = self.handle_message(message_content);
                PipelineCommResult::Ok(message_content)
            },
            _ => message
        }
    }
}
impl<T: Sharable> MessageHandler<T, Message<T>> for MultiReceiverFlags {
    fn handle_message(&mut self, message: Message<T>) -> Message<T> {
        match message {
            Message::Directive(directive) => self.handle_cp_directive(directive),
            _ => ()
        }
        
        return message;
    }
}
impl ControlPlaneHandler<()> for MultiReceiverFlags {
    fn handle_cp_directive(&mut self, directive: ControlPlaneDirective) -> () {
        match directive {
            ControlPlaneDirective::Kill => {self.kill_flag = true; self.run_flag = false;},
            ControlPlaneDirective::Stop => {self.run_flag = false;},
            ControlPlaneDirective::Start => {self.run_flag = true;},
        }
    }
}


pub enum NodeCommunicator<I: Sharable, O: Sharable> {
    SISO(WrappedReceiver<I, false>, WrappedSender<O>),
    SIMO(WrappedReceiver<I, false>, MultiSender<O>),
    MISO(MultiReceiver<I>, WrappedSender<O>)
}
impl<I: Sharable, O: Sharable> NodeCommunicator<I, O> {
    pub async fn receive(&mut self) -> Vec<PipelineCommResult<Message<I>>> {
        match self {
            NodeCommunicator::SISO(receiver, _) |
            NodeCommunicator::SIMO(receiver, _) => {
                vec![receiver.recv().await]
            },
            NodeCommunicator::MISO(receiver, _) => {
                receiver.extract_data().await
            }
        }
    }
    
    pub async fn send(&mut self, mut data: Vec<O>) -> Vec<PipelineCommResult<()>> {
        match self {
            NodeCommunicator::SISO(_, sender) |
            NodeCommunicator::MISO(_, sender) => {
                vec![sender.send(data.pop().unwrap()).await]
            }
            NodeCommunicator::SIMO(_, sender) => {
                sender.send_all(data).await
            }
        }
    }
}