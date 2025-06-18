use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::pipeline::pipeline::RadioPipeline;
use super::pipeline_thread::PipelineThread;

pub trait PipelineStep<I: Send, O: Send> : Send {
    fn run(&mut self, input: I) -> O;
}

pub struct PipelineNode<I: Send + Clone + Debug, O: Send + Clone + Debug> {
    pub input: Option<Receiver<I>>, // dump these into rwlocks so that they can be connected indpeendently of the thread operations!
    pub output: Option<Sender<O>>,
    pub id: String
}

impl<I: Send + Clone + Debug + 'static, O: Send + Clone + Debug + 'static> PipelineNode <I, O> {
    pub fn new(id: String) -> PipelineNode<I, O> {
        PipelineNode {
            input: None,
            output: None,
            id
        }
    }
    
    pub fn attach<F: Send + Clone + Debug + 'static>(mut self, id: String, step: impl PipelineStep<I, O> + 'static, pipeline: &mut RadioPipeline) -> PipelineNode<O, F> {
        let (sender, receiver) = channel::<O>();
        let mut successor: PipelineNode<O, F> = PipelineNode::new(id);
        self.output = Some(sender);
        successor.input = Some(receiver);
        
        let new_thread = PipelineThread::new(step, self);
        pipeline.nodes.push(new_thread);
        
        return successor;
    }

    pub fn call(&mut self, step: &mut impl PipelineStep<I, O>) {
        let input_data: I = self.input.as_mut().unwrap().recv().unwrap();
        let output_data: O = step.run(input_data);

        match self.output.as_mut().unwrap().send(output_data) {
            Ok(()) => {}
            Err(_msg) => {}
        }
    }
}


