use std::sync::mpsc::{Receiver, SyncSender, sync_channel, SendError, RecvTimeoutError};
use std::time::Duration;
use super::pipeline_traits::Sharable;


#[derive(Debug, Clone)]
pub enum ReceiveType<T: Sharable> {
    Single(T),
    Multi(Vec<T>),
    Dummy
}


pub struct WrappedReceiver<T: Sharable> {
    receiver: Receiver<T>,
}

impl<T: Sharable> WrappedReceiver<T> {
    pub fn new(receiver: Receiver<T>) -> Self {
        WrappedReceiver { receiver }
    }
    pub fn recv(&mut self, timeout: u64, retries: usize) -> Result<T, RecvTimeoutError> {
        Self::merciful_receive(&mut self.receiver, timeout, retries)
    }
    fn merciful_receive(receiver: &mut Receiver<T>, timeout: u64, retries: usize) -> Result<T, RecvTimeoutError> {
        let mut retry_num = 0;
        let mut success_flag = false;
        
        let mut result = Err(RecvTimeoutError::Timeout);

        while !success_flag && retry_num < retries {
            result = receiver.recv_timeout(Duration::from_millis(timeout));

            match &result {
                Err(err) => {
                    match err { RecvTimeoutError::Timeout => {retry_num += 1; continue}, _ => success_flag = true}
                },
                Ok(_) => { success_flag = true; }
            }
        };
        result
    }
}


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
    pub fn send_all(&mut self, mut data: T) -> Result<(), SendError<T>> { // all branches must be ready to receive
        let mut result = Ok(());
        
        for index in 0..self.senders.len() {
            result = self.senders[index].send(data.clone()); // evaluate this decision later
        }
        
        result
    }
    pub fn add_sender(&mut self, sender: SyncSender<T>) { 
        self.senders.push(sender);
    }
}


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
                Ok(received) => if proceed_flag { output.push(received) }
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
    SO(SyncSender<O>),
    MO(MultiSender<O>),
    Dummy
}
impl <O: Sharable> NodeSender<O> {
    pub fn send(&mut self, data: O) -> Result<(), SendError<O>> {
        match self {
            NodeSender::SO(sender) => sender.send(data),
            NodeSender::MO(sender) => sender.send_all(data),
            NodeSender::Dummy => {Err(SendError(data))}
        }
    }
}