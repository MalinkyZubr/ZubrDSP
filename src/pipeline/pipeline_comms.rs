use std::sync::mpsc::{Receiver, SyncSender, SendError, RecvTimeoutError};
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
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
    pub fn unwrap_standard(self) -> T {
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
    fn result_handler(&self, result: &mut Result<T, RecvTimeoutError>, success_flag: &mut bool, retry_num: &mut usize) {
        match result {
            Err(err) => {
                match err { RecvTimeoutError::Timeout => *retry_num += 1, _ => *success_flag = false}
            },
            Ok(_) => { *success_flag = true; }
        }
    }
    pub fn recv(&mut self, timeout: u64, retries: usize) -> Result<T, RecvTimeoutError> {
        let mut retry_num = 0;
        let mut success_flag = false;
        
        let mut result = Err(RecvTimeoutError::Timeout);

        while !success_flag && retry_num < retries {
            result = self.receiver.recv_timeout(Duration::from_millis(timeout));

            self.result_handler(&mut result, &mut success_flag, &mut retry_num);
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
    fn repeat_send(&mut self, value: T, repeats: usize, result: &mut Result<(), SendError<T>>) {
        for _ in 0..repeats {
            *result = self.sender.send(value.clone());
            match &result { Err(_) => { break }, _ => () }
        }
    }
    fn series_send(&mut self, value: Vec<T>, result: &mut Result<(), SendError<T>>) {
        for point in value {
            *result = self.sender.send(point);
            match &result { Err(_) => { break }, _ => () }
        }
    }
    pub fn send(&mut self, value: ODFormat<T>) -> Result<(), SendError<T>> {
        let mut result = Ok(());
        
        match value {
            ODFormat::Decompose(mut data_vec) => result = Err(SendError(data_vec.pop().unwrap())),
            ODFormat::Standard(value) => result = self.sender.send(value),
            ODFormat::Repeat(value, repeats) => self.repeat_send(value, repeats, &mut result),
            ODFormat::Series(value) => self.series_send(value, &mut result),
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
    fn decompose_send(&mut self, value: Vec<T>, result: &mut Result<(), SendError<T>>) {
        for (sender, point) in self.senders.iter_mut().zip(value) {
            *result = sender.send(point);
            match &result { Err(_) => { break }, _ => () }
        }
    }
    fn standard_send(&mut self, value: T, result: &mut Result<(), SendError<T>>) {
        for index in 0..self.senders.len() {
            *result = self.senders[index].send(value.clone());
            match &result { Err(_) => { break }, _ => () }
        }
    }
    fn series_send(&mut self, value: Vec<T>, result: &mut Result<(), SendError<T>>) {
        for point in value {
            for sender in self.senders.iter_mut() {
                *result = sender.send(point.clone());
                match &result { Err(_) => { break }, _ => () }
            }
        }
    }
    fn repeat_send(&mut self, value: T, repeats: usize, result: &mut Result<(), SendError<T>>) {
        for _ in 0..repeats {
            for sender in self.senders.iter_mut() {
                *result = sender.send(value.clone());
                match &result { Err(_) => { break }, _ => () }
            }
        }
    }
    pub fn send_all(&mut self, data: ODFormat<T>) -> Result<(), SendError<T>> { // all branches must be ready to receive
        let mut result = Ok(());
        
        match data {
            ODFormat::Standard(value) => self.standard_send(value, &mut result),
            ODFormat::Decompose(value) => self.decompose_send(value, &mut result),
            ODFormat::Series(value) => self.series_send(value, &mut result),
            ODFormat::Repeat(value, repeats) => self.repeat_send(value, repeats, &mut result),
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
    fn receive_handler(result: Result<T, RecvTimeoutError>, proceed_flag: &mut bool, output: &mut Vec<T>, return_value: &mut Result<ReceiveType<T>, RecvTimeoutError>) {
        match result {
            Ok(received) => { if *proceed_flag { output.push(received) }; }
            Err(error) => { *proceed_flag = false; *return_value = Err(error); }
        }
    }
    pub fn receive(&mut self) -> Result<ReceiveType<T>, RecvTimeoutError> {
        let mut output = Vec::with_capacity(self.receivers.len());
        let mut proceed_flag = true;
        let mut return_value = Ok(ReceiveType::Multichannel(Vec::new()));
        
        for receiver in self.receivers.iter_mut() {
            let received = receiver.recv(self.timeout, self.retries);
            Self::receive_handler(received, &mut proceed_flag, &mut output, &mut return_value);
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
        match self.senders.get_mut(self.channel.load(Ordering::Acquire)) {
            Some(sender) => Self::send_logic(sender, input),
            None => Err(SendError(self.index_error_unwrap(input)))
        }
    }
    fn repeat_send(selected_sender: &mut SyncSender<T>, value: T, repeats: usize, result: &mut Result<(), SendError<T>>) {
        for _ in 0..repeats {
            *result = selected_sender.send(value.clone());
            match &result { Err(_) => { break }, _ => () }
        }
    }
    fn series_send(selected_sender: &mut SyncSender<T>, value: Vec<T>, result: &mut Result<(), SendError<T>>) {
        for unit in value  {
            *result = selected_sender.send(unit);
            match &result { Err(_) => { break }, _ => () }
        }
    }
    fn send_logic(selected_sender: &mut SyncSender<T>, input: ODFormat<T>) -> Result<(), SendError<T>> {
        let mut result = Ok(());
        
        match input {
            ODFormat::Standard(value) => result = selected_sender.send(value),
            ODFormat::Repeat(value, repeats) => Self::repeat_send(selected_sender, value, repeats, &mut result),
            ODFormat::Decompose(_) => panic!("ODFormat Decompose not compatible with multiplexer"),
            ODFormat::Series(value) => Self::series_send(selected_sender, value, &mut result),
        }
        
        result
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
    channel: Arc<AtomicUsize>, // external control for the channel selection
    timeout: u64,
    retries: usize,
}
impl<T: Sharable> Demultiplexer<T> {
    pub fn new(channel: Arc<AtomicUsize>, timeout: u64, retries: usize) -> Demultiplexer<T> {
        Demultiplexer { receivers: Vec::new(), channel, timeout, retries }
    }
    pub fn receive(&mut self) -> Result<ReceiveType<T>, RecvTimeoutError> {
        match self.receivers.get_mut(self.channel.load(Ordering::Acquire)) {
            Some(receiver) => { match receiver.recv(self.timeout, self.retries) {
                Ok(received) => Ok(ReceiveType::Single(received)),
                Err(err) => Err(err)
            }},
            None => Err(RecvTimeoutError::Disconnected)
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