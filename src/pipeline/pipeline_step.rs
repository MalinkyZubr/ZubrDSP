use std::fmt::Debug;
use std::sync::mpsc;
use std::sync::mpsc::{channel, Receiver, Sender, RecvError, SendError, SyncSender, RecvTimeoutError};
use crate::pipeline::pipeline::RadioPipeline;
use super::pipeline_thread::PipelineThread;
use super::errors::PipelineError;
use super::pipeline_traits::{Sharable, Unit, HasID, Source, Sink};
use super::pipeline_comms::{RealReceiver, RealSender, NodeCommunicator, MultiReceiver, MultiSender};


pub trait PipelineStep<I: Send, O: Send> : Send {
    fn run(&mut self, input: I) -> O;
}

pub trait CallableNode<I: Sharable, O: Sharable> : Send {
    fn call(&mut self, step: &mut impl PipelineStep<I, O>) -> PipelineError;
}


pub struct PipelineNode<I: Sharable, O: Sharable> {
    pub communicator: NodeCommunicator<I, O>,
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

        let received_result = self.receive();
        match received_result.1 {
            PipelineError::Ok => (),
            _ => return received_result.1
        };

        //println!("ID: {} received!", &self.id);
        let output_data: O = step.run(received_result.0.unwrap());
        
        self.send_single(output_data)
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
