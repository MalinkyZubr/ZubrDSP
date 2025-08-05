use std::sync::mpsc::{Receiver, SyncSender, sync_channel, SendError, RecvTimeoutError};
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use futures::SinkExt;
use super::pipeline_traits::{Sharable, HasDefault};


#[derive(Debug, Clone)]
pub enum ReceiveType<T: Sharable> {
    /* 
    This is a very important enum!
    Single: Only a single value was received
    Reassembled: Multiple values were received and they were reassembled from a series sender somewhere previously
    Multichannel: This data is the aggregation of data received from multiple channels simultaneously
    Dummy: this channel is not configured
    
    The pipeline step must handle how the internal behavior responds to each of these types!
     */
    Single(T),
    Reassembled(Vec<T>),
    Multichannel(Vec<T>),
    Dummy
}


#[derive(Debug, Clone)]
pub enum ODFormat<T: Sharable> { // Output Data Format
    /*
    This is a very important enum!
    When you return data from a pipeline step, this defines how the pipeline treats that data. What do you want the pipeline to do with this data?
    Decompose: 
        Single Out Behavior: Errors, decomposition only supported for multi out
        Multi Out Behavior: Gives a vector to the multi sender, each element of the vector is sent to a separate channel
            Example: 2 channel audio data is de-Decompose, separated into 2 vectors, and processed by different branches of the pipeline
    Series:
        Single Out Behavior: Iterates over the vector from start to finish sending elements to a single consumer in sequence
            Example: I am performing an overlap add convolution on some data, and need to break it into chunks. I chunk it into a vector of vectors, 
            and return it so the sender sends each chunk separately in sequence
        Multiple Out Behavior: Replicates above behavior but round robin on multiple channels (eg, element 0 goes to channel 0, then channel 1, then element 1 goes to channel 0 etc)
    Repeat:
        Resends the same data multiple times (dont know why you would want this but I put it here all the same)
    Standard:
        Single Out Behavior: Just sends the data as is once
        Multiple Out Behavior: Just sends the data as is once but to many channels 
        
    In general: multiplexer holds exact same behavior as the single out
    */
    Decompose(Vec<T>),
    Series(Vec<T>),
    Repeat(T, usize),
    Standard(T)
}
impl<T: Sharable> ODFormat<T> {
    pub fn unwrap_Standard(self) -> T {
        match self {
            ODFormat::Standard(x) => x,
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
    pub fn send(&mut self, value: ODFormat<T>) -> Result<(), SendError<T>> {
        let mut result = Ok(());
        
        match value {
            ODFormat::Decompose(mut data_vec) => result = Err(SendError(data_vec.pop().unwrap())),
            ODFormat::Standard(value) => result = self.sender.send(value),
            ODFormat::Repeat(value, repeats) => {
                for _ in 0..*repeats {
                    result = self.sender.send(value.clone());
                    match &result { Err(err) => { break }, _ => () }
                }
            },
            ODFormat::Series(value) => {
                for point in value {
                    result = self.sender.send(point);
                    match &result { Err(err) => { break }, _ => () }
                }
            }
        }
        
        result
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

pub struct MultichannelSender<T: Sharable> {
    senders: Vec<SyncSender<T>>,
}
impl<T: Sharable> MultichannelSender<T> {
    pub fn new() -> MultichannelSender<T> {
        MultichannelSender {
            senders: Vec::new(),
        }
    }
    pub fn send_all(&mut self, mut data: ODFormat<T>) -> Result<(), SendError<T>> { // all branches must be ready to receive
        let mut result = Ok(());
        
        match data {
            ODFormat::Standard(value) => {
                for index in 0..self.senders.len() {
                    result = self.senders[index].send(value.clone());
                    match &result { Err(err) => { break }, _ => () }
                }
            }
            ODFormat::Decompose(mut value) => { // you receive an explicit condiiton from the step that the data is Decompose and each piece of data should be sent uniquely somewhere different
                for (point, sender) in value.iter().zip(self.senders.iter_mut()) {
                    result = sender.send(point);
                    match &result { Err(err) => { break }, _ => () }
                }
            },
            ODFormat::Series(value) => {
                for point in value {
                    for sender in self.senders.iter_mut() {
                        result = sender.send(point.clone());
                        match &result { Err(err) => { break }, _ => () }
                    }
                }
            }
            ODFormat::Repeat(value, repeats) => {
                for _ in 0..*repeats {
                    for sender in self.senders.iter_mut() {
                        result = sender.send(value.clone());
                        match &result { Err(err) => { break }, _ => () }
                    }
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
pub struct MultichannelReceiver<T: Sharable> {
    receivers: Vec<WrappedReceiver<T>>,
    timeout: u64,
    retries: usize,
}

impl<T: Sharable> MultichannelReceiver<T> {
    pub fn new(timeout: u64, retries: usize) ->  Self {
        MultichannelReceiver { receivers: Vec::new(), timeout, retries }
    }    
    pub fn receive(&mut self) -> Result<ReceiveType<T>, RecvTimeoutError> {
        let mut output = Vec::with_capacity(self.receivers.len());
        let mut proceed_flag = true;
        let mut return_value = Ok(ReceiveType::Multichannel(Vec::new()));
        
        for receiver in self.receivers.iter_mut() {
            let received = receiver.recv(self.timeout, self.retries);
            match received {
                Ok(received) => { if proceed_flag { output.push(received) }; }
                Err(error) => { proceed_flag = false; return_value = Err(error); }
            }
        }
        
        if output.len() == self.receivers.len() {
            return_value = Ok(ReceiveType::Multichannel(output));
        }
        
        return_value
    }
    pub fn add_receiver(&mut self, receiver: WrappedReceiver<T>) {
        self.receivers.push(receiver);
    }
}


#[derive(Debug)]
pub struct Multiplexer<T: Sharable> {
    senders: Vec<SyncSender<T>>,
    channel: Arc<AtomicUsize> // external control for the channel selection
}
impl<T: Sharable> Multiplexer<T> {
    pub fn new(channel: Arc<AtomicUsize>) -> Multiplexer<T> {
        Multiplexer { senders: Vec::new(), channel }
    }
    pub fn send(&mut self, input: ODFormat<T>) -> Result<(), SendError<T>> {
        match self.senders.get(self.channel.load(Ordering::Acquire)) {
            Ok(sender) => self.send_logic(sender, input),
            Err(_) => Err(self.index_error_unwrap(input))
        }
    }
    fn send_logic(&mut self, selected_sender: &mut SyncSender<T>, input: ODFormat<T>) -> Result<(), SendError<T>> {
        match input {
            ODFormat::Standard(value) => selected_sender.send(value),
            ODFormat::Repeat(value, repeats) => {
                for _ in 0..*repeats {
                    selected_sender.send(value.clone())
                }
            },
            ODFormat::Decompose(_) => panic!("ODFormat Decompose not compatible with multiplexer"),
            ODFormat::Series(value) => {
                for unit in value  {
                    selected_sender.send(unit)
                }
            }
        }
    }
    fn index_error_unwrap(&self, input: ODFormat<T>) -> T {
        match input {
            ODFormat::Standard(result) | ODFormat::Repeat(result, _) => result,
            ODFormat::Series(mut result) | ODFormat::Decompose(mut result) => result.pop().unwrap(),
        }
    }

    pub fn add_sender(&mut self, sender: SyncSender<T>) {
        self.senders.push(sender);
    }
}

#[derive(Debug)]
pub struct Demultiplexer<T: Sharable> {
    receivers: Vec<WrappedReceiver<T>>,
    channel: Arc<AtomicUsize> // external control for the channel selection
}
impl<T: Sharable> Demultiplexer<T> {
    pub fn new(channel: Arc<AtomicUsize>) -> Demultiplexer<T> {
        Demultiplexer { receivers: Vec::new(), channel }
    }
    pub fn receive(&mut self) -> Result<ReceiveType<T>, RecvTimeoutError> {
        match self.receivers.get(self.channel.load(Ordering::Acquire)) {
            Ok(receiver) => { match receiver.recv() {
                Ok(received) => Ok(ReceiveType::Single(received))
            }},
            Err(_) => Err(RecvTimeoutError::Disconnected)
        }
    }
    pub fn add_receiver(&mut self, receiver: WrappedReceiver<T>) {
        self.receivers.push(receiver);
    }
}


#[derive(Debug)]
pub struct Reassembler<T: Sharable> {
    receiver: WrappedReceiver<T>,
    num_receives: usize,
    timeout: u64,
    retries: usize
}
impl<T: Sharable> Reassembler<T> {
    pub fn new(receiver: WrappedReceiver<T>, num_receives: usize, timeout: u64, retries: usize) -> Reassembler<T> {
        Self { receiver, num_receives, timeout, retries }
    }
    
    pub fn receive(&mut self) -> Result<ReceiveType<T>, RecvTimeoutError> {
        let mut receive_vec = Vec::with_capacity(self.num_receives);
        
        for _ in 0..self.num_receives {
            match self.receiver.recv(self.timeout, self.retries) {
                Ok(received) => receive_vec.push(received),
                Err(error) => return Err(error)
            }
        }
        
        Ok(ReceiveType::Reassembled(receive_vec))
    }
}


#[derive(Debug)]
pub enum NodeReceiver<I: Sharable> {
    SI(SingleReceiver<I>),
    MI(MultichannelReceiver<I>),
    REA(Reassembler<I>),
    DMI(Demultiplexer<I>),
    Dummy
}
impl<I: Sharable> NodeReceiver<I> {
    pub fn receive(&mut self) -> Result<ReceiveType<I>, RecvTimeoutError> {
        match self {
            NodeReceiver::SI(receiver) => receiver.receive(),
            NodeReceiver::MI(receiver) => receiver.receive(),
            NodeReceiver::REA(receiver) => receiver.receive(),
            NodeReceiver::DMI(receiver) => receiver.receive(),
            NodeReceiver::Dummy => Ok(ReceiveType::Dummy)
        }
    }
}

pub enum NodeSender<O: Sharable> {
    SO(SingleSender<O>),
    MO(MultichannelSender<O>),
    MUO(Multiplexer<O>),
    Dummy
}
impl <O: Sharable> NodeSender<O> {
    pub fn send(&mut self, data: ODFormat<O>) -> Result<(), SendError<O>> {
        match self {
            NodeSender::SO(sender) => sender.send(data),
            NodeSender::MO(sender) => sender.send_all(data),
            NodeSender::MUO(sender) => sender.send(data),
            NodeSender::Dummy => {Err(SendError(match data { ODFormat::Standard(val) => val, _ => panic!("How this even happens?")}))}
        }
    }
}