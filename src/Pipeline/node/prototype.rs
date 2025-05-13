use std::fmt::Debug;

use super::messages::{Source, Sink};

pub trait PipelineStep<I: Send + 'static, O: Send + 'static> : Send {
    fn run<'a>(&mut self, input: I) -> O;
}

pub struct PipelineNode<I: Send + Clone + 'static + Debug, O: Send + Clone + 'static + Debug> {
    step: Box<dyn PipelineStep<I, O>>,
    input: Option<Box<dyn Source<I>>>,
    output: Option<Box<dyn Sink<O>>>,
}

impl<I: Send + Clone + 'static + Debug, O: Send + Clone + 'static + Debug> PipelineNode <I, O> {
    pub fn new(step: Box<dyn PipelineStep<I, O>>) -> PipelineNode<I, O> {
        PipelineNode {
            step,
            input: None,
            output: None,
        }
    }
}

pub trait PipelineNodeGeneric {
    fn call(&mut self); // have this pass errors
}

impl<I: Send + Clone + 'static + Debug, O: Send + Clone + 'static + Debug> PipelineNodeGeneric for PipelineNode<I, O> { // for tasks like viterbi algorithm, context is needed
    fn call(&mut self) {
        let input_data: I = self.input.as_mut().unwrap().recv().unwrap();
        let output_data: O = self.step.run(input_data);

        match self.output.as_mut().unwrap().send(output_data) {
            Ok(()) => {}
            Err(_msg) => {}
        }
    }
}

impl<I: Send + Clone + 'static + Debug, O: Send + Clone + 'static + Debug> PipelineNode<I, O> {
    pub fn set_input(&mut self, input: Box<dyn Source<I>>) {
        self.input = Some(input);
    }

    pub fn set_output(&mut self, output: Box<dyn Sink<O>>) {
        self.output = Some(output);
    }
}

