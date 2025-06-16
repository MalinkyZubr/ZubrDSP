use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};

pub trait PipelineStep<I: Send, O: Send> : Send {
    fn run(&mut self, input: I) -> O;
}

pub struct PipelineNode<I: Send + Clone + Debug, O: Send + Clone + Debug> {
    pub input: Option<Receiver<I>>, // dump these into rwlocks so that they can be connected indpeendently of the thread operations!
    pub output: Option<Sender<O>>,
}

impl<I: Send + Clone + Debug, O: Send + Clone + Debug> PipelineNode <I, O> {
    pub fn new() -> PipelineNode<I, O> {
        PipelineNode {
            input: None,
            output: None,
        }
    }
    
    pub fn attach<F: Send + Clone + Debug>(&mut self, successor: &mut PipelineNode<O, F>) {
        let (sender, receiver) = channel::<O>();
        self.output = Some(sender);
        successor.input = Some(receiver);
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
