use std::sync::mpsc::{Receiver, SyncSender, sync_channel, SendError, RecvTimeoutError};
use std::time::Duration;
use super::pipeline_traits::{Sharable, HasDefault};


#[derive(Debug, Clone)]
pub enum ReceiveType<T: Sharable> {
    Single(T),
    Multi(Vec<T>),
    Dummy
}


#[derive(Debug, Clone)]
pub enum SendType<T: Sharable> {
    Interleaved(Vec<T>),
    NonInterleaved(T)
}
impl<T: Sharable> SendType<T> {
    pub fn unwrap_noninterleaved(self) -> T {
        match self {
            SendType::NonInterleaved(x) => x,
            _ => panic!()
        }
    }
}


#[derive(Debug)]
pub struct WrappedReceiver<T: Sharable> {
    receiver: Receiver<T>,
    feedback_startup_flag: bool
}
impl<T: Sharable + HasDefault> WrappedReceiver<T> {
    pub fn new(receiver: Receiver<T>) -> Self {
        WrappedReceiver { receiver, feedback_startup_flag: false }
    }
    pub fn set_startup_flag(mut self) -> Self {
        self.feedback_startup_flag = true;
        self
    }
    pub fn recv(&mut self, timeout: u64, retries: usize) -> Result<T, RecvTimeoutError> {
        let mut retry_num = 0;
        let mut success_flag = false;
        
        let mut result = Err(RecvTimeoutError::Timeout);

        while !success_flag && retry_num < retries {
            result = self.receiver.recv_timeout(Duration::from_millis(timeout));

            match &mut result {
                Err(err) => {
                    match err { RecvTimeoutError::Timeout => {retry_num += 1; continue}, _ => success_flag = true}
                },
                Ok(_) => { success_flag = true; }
            }
        };
        if self.feedback_startup_flag {
            self.feedback_startup_flag = false;
            Ok(T::default())
        }
        else { result }
    }
}

#[derive(Debug)]
pub struct SingleSender<T: Sharable> {
    sender: SyncSender<T>,
}
impl<T: Sharable> SingleSender<T> {
    pub fn new(sender: SyncSender<T>) -> Self {
        SingleSender { sender }
    }
    pub fn send(&mut self, value: SendType<T>) -> Result<(), SendError<T>> {
        match value {
            SendType::Interleaved(mut data_vec) => Err(SendError(data_vec.pop().unwrap())),
            SendType::NonInterleaved(value) => self.sender.send(value)
        }
    }
}

#[derive(Debug)]
pub struct SingleReceiver<T: Sharable> {
    receiver: WrappedReceiver<T>,
    timeout: u64,
    retries: usize
}
impl<T: Sharable> SingleReceiver<T> {
    pub fn new(receiver: WrappedReceiver<T>, timeout: u64, retries: usize) -> SingleReceiver<T> {
        SingleReceiver { receiver, timeout,  retries }
    }
    pub fn receive(&mut self) -> Result<ReceiveType<T>, RecvTimeoutError> {
        match self.receiver.recv(self.timeout, self.retries) {
            Ok(result) => Ok(ReceiveType::Single(result)),
            Err(err) => Err(err)
        }
    }
    pub fn extract_receiver(self) -> WrappedReceiver<T> {
        self.receiver
    }
}

pub struct MultiSender<T: Sharable> {
    senders: Vec<SyncSender<T>>,
}
impl<T: Sharable> MultiSender<T> {
    pub fn new() -> MultiSender<T> {
        MultiSender {
            senders: Vec::new(),
        }
    }
    pub fn send_all(&mut self, mut data: SendType<T>) -> Result<(), SendError<T>> { // all branches must be ready to receive
        let mut result = Ok(());
        
        match data {
            SendType::NonInterleaved(value) => {
                for index in 0..self.senders.len() {
                    result = self.senders[index].send(value.clone());
                    match &result { Err(err) => { break }, _ => () }
                }
            }
            SendType::Interleaved(mut value) => { // you receive an explicit condiiton from the step that the data is interleaved and each piece of data should be sent uniquely somewhere different
                value.reverse();
                for index in 0..self.senders.len() {
                    result = self.senders[index].send(value.pop().unwrap());
                    match &result { Err(err) => { break }, _ => () }
                }
            }
        }
        
        result
    }
    pub fn add_sender(&mut self, sender: SyncSender<T>) { 
        self.senders.push(sender);
    }
}


#[derive(Debug)]
pub struct MultiReceiver<T: Sharable> {
    receivers: Vec<WrappedReceiver<T>>,
    timeout: u64,
    retries: usize,
}

impl<T: Sharable> MultiReceiver<T> {
    pub fn new(timeout: u64, retries: usize) ->  Self {
        MultiReceiver { receivers: Vec::new(), timeout, retries }
    }    
    pub fn receive(&mut self) -> Result<ReceiveType<T>, RecvTimeoutError> {
        let mut output = Vec::with_capacity(self.receivers.len());
        let mut proceed_flag = true;
        let mut return_value = Ok(ReceiveType::Multi(Vec::new()));
        
        for receiver in self.receivers.iter_mut() {
            let received = receiver.recv(self.timeout, self.retries);
            match received {
                Ok(received) => { if proceed_flag { output.push(received) }; }
                Err(error) => { proceed_flag = false; return_value = Err(error); }
            }
        }
        
        if output.len() ==  self.receivers.len() {
            return_value = Ok(ReceiveType::Multi(output));
        }
        
        return_value
    }
    pub fn add_receiver(&mut self, receiver: WrappedReceiver<T>) {
        self.receivers.push(receiver);
    }
}


#[derive(Debug)]
pub enum NodeReceiver<I: Sharable> {
    SI(SingleReceiver<I>),
    MI(MultiReceiver<I>),
    Dummy
}
impl<I: Sharable> NodeReceiver<I> {
    pub fn receive(&mut self) -> Result<ReceiveType<I>, RecvTimeoutError> {
        match self {
            NodeReceiver::SI(receiver) => receiver.receive(),
            NodeReceiver::MI(receiver) => receiver.receive(),
            NodeReceiver::Dummy => Ok(ReceiveType::Dummy)
        }
    }
}

pub enum NodeSender<O: Sharable> {
    SO(SingleSender<O>),
    MO(MultiSender<O>),
    Dummy
}
impl <O: Sharable> NodeSender<O> {
    pub fn send(&mut self, data: SendType<O>) -> Result<(), SendError<O>> {
        match self {
            NodeSender::SO(sender) => sender.send(data),
            NodeSender::MO(sender) => sender.send_all(data),
            NodeSender::Dummy => {Err(SendError(match data { SendType::NonInterleaved(val) => val, SendType::Interleaved(_) => panic!("How this even happens?")}))}
        }
    }
}