use num::complex::Complex;

use crate::Pipeline::{buffer::BufferType};
use super::messages::{Source, Sink};


pub trait PipelineStep<DataType: BufferType> { // This is the bit that is not abstracted away. Passed to pipeline
    fn computation(&mut self, data: DataType) -> DataType;
}

pub struct PipelineNode<DataType: BufferType> {
    step: Box<dyn PipelineStep<DataType>>, // could lead to inefficiencies? switch to enum later perhaps
    input: Box<dyn Source<DataType>>,
    output: Box<dyn Sink<DataType>>
}

pub trait PipelineNodeGeneric {
    fn call(&mut self);
}

impl<DataType: BufferType> PipelineNodeGeneric for PipelineNode<DataType> {
    fn call(&mut self) {
        let input_data: DataType = self.input.recv().unwrap();
        let output_data: DataType = self.step.computation(input_data);

        match self.output.send(output_data) {
            Ok(()) => {}
            Err(_msg) => {}
        }
    }
}

impl<DataType: BufferType> PipelineNode<DataType> {
    pub fn set_input(&mut self, input: Box<dyn Source<DataType>>) {
        self.input = input;
    }

    pub fn set_output(&mut self, output: Box<dyn Sink<DataType>>) {
        self.output = output;
    }
}

pub enum PipelineNodeEnum {
    Scalar(PipelineNode<Complex<f32>>),
    Vector(PipelineNode<Vec<Complex<f32>>>)
}

impl PipelineNodeEnum {
    pub fn get_generalized(&mut self) -> &mut dyn PipelineNodeGeneric {
        match self {
            PipelineNodeEnum::Scalar(node) => node,
            PipelineNodeEnum::Vector(node) => node
        }
    }
}