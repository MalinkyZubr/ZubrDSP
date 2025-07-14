use std::fmt::Debug;
use std::sync::mpsc;
use std::sync::mpsc::{channel, Receiver, Sender, RecvError, SendError, SyncSender, RecvTimeoutError};
use crate::pipeline::pipeline::RadioPipeline;
use super::pipeline_thread::PipelineThread;


pub enum Assert<const CHECK: bool> {}
pub trait IsTrue {}
impl IsTrue for Assert<true> {}


pub trait Source {}
pub trait Sink {}

pub trait Sharable = Send + Sync + Debug + Copy + 'static;
pub trait Unit: Send + Clone {
    fn gen() -> Self;
}
impl Unit for () {
    fn gen() -> Self {
        ()
    }
}

pub enum PipelineError {
    SendError,
    ReceiveError,
    Timeout,
    Ok
}

enum RealSender<T: Sharable> {
    Real(SyncSender<Vec<T>>),
    Dummy
}
impl<T: Sharable> RealSender<T> {
    pub fn send(&mut self, data: Vec<T>) -> Result<(), SendError<Vec<T>>> {
        match self {
            RealSender::Real(sender) => sender.send(data),
            RealSender::Dummy => Result::Ok(())
        }
    }
}

enum RealReceiver<T: Sharable> {
    Real(Receiver<Vec<T>>),
    Dummy
}
impl<T: Sharable> RealReceiver<T> {
    pub fn recv(&mut self) -> (Result<Vec<T>, RecvError>, bool) {
        match self {
            RealReceiver::Real(receiver) => (receiver.recv(), true),
            RealReceiver::Dummy => (Result::Err(RecvError), false)
        }
    }

    pub fn recv_timeout(&mut self, timeout: core::time::Duration) -> (Result<Vec<T>, RecvTimeoutError>, bool) {
        match self {
            RealReceiver::Real(receiver) => (receiver.recv_timeout(timeout), true),
            RealReceiver::Dummy => (Result::Err(RecvTimeoutError::Timeout), false)
        }
    }
}

const fn valid_io_ratio(ni: usize, no: usize) -> bool {
    ni == 1 || no == 1
}

pub trait PipelineStep<I: Send, O: Send, const NI: usize = 1, const NO: usize = 1> : Send
where
    Assert<{valid_io_ratio(NI, NO)}>: IsTrue 
{
    fn run(&mut self, input: Vec<I>) -> Vec<O>;
}

pub trait CallableNode<I: Sharable, O: Sharable> : Send {
    fn call(&mut self, step: &mut impl PipelineStep<I, O>) -> PipelineError;
} 

pub trait HasID {
    fn get_id(&self) -> String;
    fn set_id(&mut self, id: String);
}

pub struct PipelineNode<I: Sharable, O: Sharable, const NI: usize, const NO: usize>
where
    Assert<{valid_io_ratio(NI, NO)}>: IsTrue 
{
    pub input: RealReceiver<I>,
    pub output: RealSender<O>,
    pub id: String,
    input_buffer: Vec<Vec<I>>,
    output_buffer: Vec<Vec<O>>
}

impl<I: Sharable, O: Sharable, const NI: usize, const NO: usize> HasID for PipelineNode<I, O, NI, NO>
where
    Assert<{valid_io_ratio(NI, NO)}>: IsTrue
{
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }
}

impl<I: Sharable, O: Sharable, const NI: usize, const NO: usize> CallableNode<I, O> for PipelineNode<I, O, NI, NO>
where
    Assert<{valid_io_ratio(NI, NO)}>: IsTrue
{
    fn call(&mut self, step: &mut impl PipelineStep<I, O>) -> PipelineError {
        //println!("ID: {} received!", &self.id);
        let output_data: Vec<O> = step.run(input_data);

    }
}

impl<I: Sharable, O: Sharable, const NI: usize, const NO: usize> PipelineNode<I, O, NI, NO>
where
    Assert<{valid_io_ratio(NI, NO)}>: IsTrue
{
    pub fn new() -> PipelineNode<I, O, NI, NO> {
        PipelineNode {
            input: RealReceiver::Dummy,
            output: RealSender::Dummy,
            id: String::from(""),
            input_buffer: vec![Vec::new(); NI],
            output_buffer: vec![Vec::new(); NO]
        }
    }

    fn siso_call(&mut self, step: &mut impl PipelineStep<I, O>) -> PipelineError {
        let mut input_data = self.receive();
        match input_data.1 {
            PipelineError::Ok => (),
            _ => return input_data.1
        };

        let output_data = step.run(input_data.0.unwrap());
        self.send(output_data)
    }

    fn miso_call(&mut self, step: &mut impl PipelineStep<I, O>) -> PipelineError {
        let mut input_index = 0;
        
        while input_index < NI {
            let input_value = self.receive();
            match input_value.1 {
                PipelineError::Ok => (),
                _ => continue
            };
            
            self.input_buffer[input_index] = input_value.0.unwrap();
        }
        
        
    }
    
    fn simo_call(&mut self, step: &mut impl PipelineStep<I, O>) -> PipelineError {

    }
    
    fn receive(&mut self) -> (Option<Vec<I>>, PipelineError) { // make interruption of the process an async process, dont use timeouts. Wastes compute
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

    fn send(&mut self, data: Vec<O>) -> PipelineError {
        let return_error = match self.output.send(data) {
            Ok(()) => PipelineError::Ok,
            Err(_msg) => PipelineError::SendError
        };

        return return_error;
    }

    pub fn attach<F: Sharable, const NIF: usize, const NOF: usize>(mut self, id: String, step: impl PipelineStep<I, O> + 'static, pipeline: &mut RadioPipeline) -> PipelineNode<O, F, NIF, NOF> 
    where
        Assert<{valid_io_ratio(NIF, NOF)}>: IsTrue
    {
        let (sender, receiver) = mpsc::sync_channel::<Vec<O>>(1);
        let mut successor: PipelineNode<O, F, NIF, NOF> = PipelineNode::new();

        self.set_id(id);

        self.output = RealSender::Real(sender);
        successor.input = RealReceiver::Real(receiver);

        let new_thread = PipelineThread::new(step, self);
        pipeline.nodes.push(new_thread);

        return successor;
    }

    pub fn cap_pipeline(mut self, id: String, step: impl PipelineStep<I, O> + 'static + Sink, pipeline: &mut RadioPipeline)
    where O: Unit {
        self.set_id(id);

        self.output = RealSender::Dummy;

        let new_thread = PipelineThread::new(step, self);
        pipeline.nodes.push(new_thread);
    }

    pub fn start_pipeline<F: Sharable, const NIF: usize, const NOF: usize>(start_id: String, source_step: impl PipelineStep<I, O> + 'static + Source, pipeline: &mut RadioPipeline) -> PipelineNode<O, F>
    where 
        I: Unit,
        Assert<{valid_io_ratio(NIF, NOF)}>: IsTrue
    {
        let (sender, receiver) = mpsc::sync_channel::<Vec<O>>(1);

        let start_node: PipelineNode<I, O, NI, NO> = PipelineNode { input: RealReceiver::Dummy, output: RealSender::Real(sender), id: start_id };

        let mut successor: PipelineNode<O, F, NIF, NOF> = PipelineNode::new();
        successor.input = RealReceiver::Real(receiver);

        let new_thread = PipelineThread::new(source_step, start_node);
        pipeline.nodes.push(new_thread);

        return successor;
    }
}
