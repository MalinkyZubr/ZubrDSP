use std::fmt::Debug;
use std::sync::mpsc;
use std::sync::mpsc::{channel, Receiver, Sender, RecvError, SendError, SyncSender, RecvTimeoutError};
use crate::pipeline::pipeline::RadioPipeline;
use super::pipeline_thread::PipelineThread;


pub trait Source {}
pub trait Sink {}

pub trait Sharable = Send + Sync + Debug + 'static;
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
    Ok
}

enum RealSender<T: Sharable> {
    Real(SyncSender<T>),
    Dummy
}
impl<T: Sharable> RealSender<T> {
    pub fn send(&mut self, data: T) -> Result<(), SendError<T>> {
        match self {
            RealSender::Real(sender) => sender.send(data),
            RealSender::Dummy => Result::Ok(())
        }
    }
}

enum RealReceiver<T: Sharable> {
    Real(Receiver<T>),
    Dummy
}
impl<T: Sharable> RealReceiver<T> {
    pub fn recv(&mut self) -> (Result<T, RecvError>, bool) {
        match self {
            RealReceiver::Real(receiver) => (receiver.recv(), true),
            RealReceiver::Dummy => (Result::Err(RecvError), false)
        }
    }

    pub fn recv_timeout(&mut self, timeout: core::time::Duration) -> (Result<T, RecvTimeoutError>, bool) {
        match self {
            RealReceiver::Real(receiver) => (receiver.recv_timeout(timeout), true),
            RealReceiver::Dummy => (Result::Err(RecvTimeoutError::Timeout), false)
        }
    }
}

pub trait PipelineStep<I: Send, O: Send> : Send {
    fn run(&mut self, input: Option<I>) -> O;
}

pub trait CallableNode<I: Sharable, O: Sharable> : Send {
    fn call(&mut self, step: &mut impl PipelineStep<I, O>) -> PipelineError;
} 

pub trait HasID {
    fn get_id(&self) -> String;
    fn set_id(&mut self, id: String);
}

pub struct PipelineNode<I: Sharable, O: Sharable> {
    pub input: RealReceiver<I>,
    pub output: RealSender<O>,
    pub id: String
}

impl<I: Sharable, O: Sharable> HasID for PipelineNode<I, O> {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }
}

impl<I: Sharable, O: Sharable> CallableNode<I, O> for PipelineNode<I, O> {
    fn call(&mut self, step: &mut impl PipelineStep<I, O>) -> PipelineError {
        //println!("ID: {} waiting for input!", &self.id);
        let input_data;

        match self.input.recv_timeout(std::time::Duration::from_millis(100)) { // add some iterators here to add multi in-multi out functionality
            (Ok(data), _) => input_data = Some(data),
            (Err(RecvTimeoutError), true) => {
                //println!("ID: {} failed to receive!", &self.id);
                return PipelineError::ReceiveError
            }
            (Err(RecvTimeoutError), false) => input_data = None
        }

        //println!("ID: {} received!", &self.id);
        let output_data: O = step.run(input_data);

        let return_error = match self.output.send(output_data) {
            Ok(()) => PipelineError::Ok,
            Err(_msg) => PipelineError::SendError
        };

        //println!("ID: {} sent!", &self.id);

        return return_error;
    }
}


impl<I: Sharable, O: Sharable> PipelineNode<I, O> {
    pub fn new() -> PipelineNode<I, O> {
        PipelineNode {
            input: RealReceiver::Dummy,
            output: RealSender::Dummy,
            id: String::from("")
        }
    }

    pub fn attach<F: Sharable>(mut self, id: String, step: impl PipelineStep<I, O> + 'static, pipeline: &mut RadioPipeline) -> PipelineNode<O, F> {
        let (sender, receiver) = mpsc::sync_channel::<O>(1);
        let mut successor: PipelineNode<O, F> = PipelineNode::new();

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

    pub fn start_pipeline<F: Sharable>(start_id: String, source_step: impl PipelineStep<I, O> + 'static + Source, pipeline: &mut RadioPipeline) -> PipelineNode<O, F>
    where I: Unit {
        let (sender, receiver) = mpsc::sync_channel::<O>(1);

        let start_node: PipelineNode<I, O> = PipelineNode { input: RealReceiver::Dummy, output: RealSender::Real(sender), id: start_id };

        let mut successor: PipelineNode<O, F> = PipelineNode::new();
        successor.input = RealReceiver::Real(receiver);

        let new_thread = PipelineThread::new(source_step, start_node);
        pipeline.nodes.push(new_thread);

        return successor;
    }
}
