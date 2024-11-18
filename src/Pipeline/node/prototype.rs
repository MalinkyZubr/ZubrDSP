use num::complex::Complex;
use std::sync::{Arc, Mutex};

use crate::Pipeline::{buffer::BufferType};
use super::messages::{Source, Sink};


pub type PipelineStep<T> = dyn Fn(T) -> T + Send + 'static;

pub struct PipelineNode<DataType: BufferType> {
    step: Box<PipelineStep<DataType>>,
    input: Option<Box<dyn Source<DataType>>>,
    output: Option<Box<dyn Sink<DataType>>>
}

impl<DataType: BufferType> PipelineNode <DataType> {
    pub fn new(step: Box<PipelineStep<DataType>>) -> PipelineNode<DataType> {
        PipelineNode {
            step,
            input: None,
            output: None
        }
    }
}

pub trait PipelineNodeGeneric {
    fn call(&mut self); // have this pass errors
}

impl<DataType: BufferType> PipelineNodeGeneric for PipelineNode<DataType> {
    fn call(&mut self) {
        let input_data: DataType = self.input.as_mut().unwrap().recv().unwrap();
        let output_data: DataType = (self.step)(input_data);

        match self.output.as_mut().unwrap().send(output_data) {
            Ok(()) => {}
            Err(_msg) => {}
        }
    }
}

impl<DataType: BufferType> PipelineNode<DataType> {
    pub fn set_input(&mut self, input: Box<dyn Source<DataType>>) {
        self.input = Some(input);
    }

    pub fn set_output(&mut self, output: Box<dyn Sink<DataType>>) {
        self.output = Some(output);
    }
}

