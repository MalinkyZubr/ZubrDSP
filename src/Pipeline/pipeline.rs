use num::complex::Complex;
use std::thread;

use super::buffer::{FrequencyToTimeAdapter, TimeToFrequencyAdapter, NonConvertingAdapter};
use super::node::prototype::{PipelineNode, PipelineNodeEnum};
use super::node::messages::{Source, Sink};


pub struct Pipeline<ThreadType> {
    source: Box<dyn Source<Complex<f32>>>,
    sink: Box<dyn Sink<Complex<f32>>>,
    freq_buffsize: usize,
    thread_pool: Vec<thread::JoinHandle<ThreadType>>,
    node_pool: Vec<PipelineNodeEnum>
}


impl<ThreadType> Pipeline<ThreadType> {
    pub fn add_time_domain_step(&mut self, node: PipelineNode<Complex<f32>>) {
        let node_enum: PipelineNodeEnum = PipelineNodeEnum::TimeDomain(node);
        self.node_pool.push(node_enum);
    }

    pub fn add_frequency_domain_step(&mut self, node: PipelineNode<Vec<Complex<f32>>>) {
        let node_enum: PipelineNodeEnum = PipelineNodeEnum::FrequencyDomain(node);
        self.node_pool.push(node_enum);
    }

    fn generate_threads(&mut self) {}

    pub fn run(&mut self) {}

    pub fn pause(&mut self) {}

    pub fn end(&mut self) {}
}