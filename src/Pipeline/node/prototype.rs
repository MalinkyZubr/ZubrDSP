use std::fmt::Debug;

use super::messages::{Source, Sink};

pub trait PipelineStep<T: Send + 'static> : Send {
    fn run<'a>(&mut self, input: T) -> T;
}

pub struct PipelineNode<T: Send + Clone + 'static + Debug> {
    step: Box<dyn PipelineStep<T>>,
    input: Option<Box<dyn Source<T>>>,
    output: Option<Box<dyn Sink<T>>>,
}

impl<T: Send + Clone + 'static + Debug> PipelineNode <T> {
    pub fn new(step: Box<dyn PipelineStep<T>>) -> PipelineNode<T> {
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

impl<T: Send + Clone + 'static + Debug> PipelineNodeGeneric for PipelineNode<T> { // for tasks like viterbi algorithm, context is needed
    fn call(&mut self) {
        let input_data: T = self.input.as_mut().unwrap().recv().unwrap();
        let output_data: T = self.step.run(input_data);

        match self.output.as_mut().unwrap().send(output_data) {
            Ok(()) => {}
            Err(_msg) => {}
        }
    }
}

impl<T: Send + Clone + 'static + Debug> PipelineNode<T> {
    pub fn set_input(&mut self, input: Box<dyn Source<T>>) {
        self.input = Some(input);
    }

    pub fn set_output(&mut self, output: Box<dyn Sink<T>>) {
        self.output = Some(output);
    }
}

