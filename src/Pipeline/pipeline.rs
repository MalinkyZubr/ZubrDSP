use num::complex::Complex;
use std::thread::{self, Thread};
use std::sync::mpsc;

use super::buffer::{BufferType, ScalarToVectorAdapter, VectorToScalarAdapter};
use super::node;
use super::node::prototype::{PipelineNode, PipelineNodeEnum, PipelineNodeGeneric};
use super::node::messages::{Source, Sink, ReceiverWrapper, SenderWrapper, create_node_connection};


pub struct PipelineThread<ThreadType> {
    node: PipelineNodeEnum,
    runflag: bool,
    join_handle: Option<thread::JoinHandle<ThreadType>>,
}

impl<ThreadType> PipelineThread<ThreadType> {
    pub fn pause(&mut self) {
        self.runflag = false;
    }
    pub fn resume(&mut self) {
        self.runflag = true;
    }
    pub fn run(&mut self) {
        
    }
    fn run_loop(&mut self) {
        let mut generalized: &mut dyn PipelineNodeGeneric = self.node.get_generalized();
        while self.runflag {
            generalized.call();
        }
    }
}

struct Welder {
    buff_size: usize
}

impl Welder {
    pub fn new(buff_size: usize) -> Welder {
        Welder {
            buff_size: buff_size
        }
    }
    pub fn weld(&self, src: &mut PipelineNodeEnum, dst: &mut PipelineNodeEnum) -> Option<Box<dyn PipelineNodeGeneric>> {
        match (src, dst) {
            (PipelineNodeEnum::Scalar(src), 
            PipelineNodeEnum::Scalar(dst)) => {self.weld_scalar(src, dst); None},

            (PipelineNodeEnum::Vector(src), 
            PipelineNodeEnum::Vector(dst)) => {self.weld_vector(src, dst); None},

            (PipelineNodeEnum::Scalar(src), 
            PipelineNodeEnum::Vector(dst)) => Some(Box::new(self.weld_scalar_to_vector(src, dst))),

            (PipelineNodeEnum::Vector(src), 
            PipelineNodeEnum::Scalar(dst)) => Some(Box::new(self.weld_vector_to_scalar(src, dst)))
        }
    }

    fn weld_scalar(&self, src: &mut PipelineNode<Complex<f32>>, dst: &mut PipelineNode<Complex<f32>>) {
        let (sender, receiver) = create_node_connection::<Complex<f32>>();
        src.set_output(Box::new(sender));
        dst.set_input(Box::new(receiver));
    }

    fn weld_vector(&self, src: &mut PipelineNode<Vec<Complex<f32>>>, dst: &mut PipelineNode<Vec<Complex<f32>>>) {
        let (sender, receiver) = create_node_connection::<Vec<Complex<f32>>>();
        src.set_output(Box::new(sender));
        dst.set_input(Box::new(receiver));
    }

    fn weld_scalar_to_vector(&self, src: &mut PipelineNode<Complex<f32>>, dst: &mut PipelineNode<Vec<Complex<f32>>>) -> ScalarToVectorAdapter {
        let (scalar_sender, scalar_receiver) = create_node_connection::<Complex<f32>>();
        let (vector_sender, vector_receiver) = create_node_connection::<Vec<Complex<f32>>>();
        
        src.set_output(Box::new(scalar_sender));
        let adapter: ScalarToVectorAdapter = ScalarToVectorAdapter::new(scalar_receiver, vector_sender, self.buff_size);
        dst.set_input(Box::new(vector_receiver));

        return adapter;
    }

    fn weld_vector_to_scalar(&self, src: &mut PipelineNode<Vec<Complex<f32>>>, dst: &mut PipelineNode<Complex<f32>>) -> VectorToScalarAdapter {
        let (scalar_sender, scalar_receiver) = create_node_connection::<Complex<f32>>();
        let (vector_sender, vector_receiver) = create_node_connection::<Vec<Complex<f32>>>();
        
        src.set_output(Box::new(vector_sender));
        let adapter: VectorToScalarAdapter = VectorToScalarAdapter::new(vector_receiver, scalar_sender, self.buff_size);
        dst.set_input(Box::new(scalar_receiver));

        return adapter;
    }
}

pub struct Pipeline<ThreadType> {
    source: Box<dyn Source<Complex<f32>>>,
    sink: Box<dyn Sink<Complex<f32>>>,
    buff_size: usize,
    thread_pool: Vec<thread::JoinHandle<ThreadType>>,
    node_pool: Vec<PipelineNodeEnum>,
    welder: Welder
}

impl<ThreadType> Pipeline<ThreadType> {
    pub fn new(buff_size: usize, source: Box<dyn Source<Complex<f32>>>, sink:Box<dyn Sink<Complex<f32>>>) -> Pipeline<ThreadType> {
        Pipeline {
            source,
            sink,
            buff_size,
            thread_pool: Vec::new(),
            node_pool: Vec::new(),
            welder: Welder {buff_size}
        }
    }
    pub fn add_time_domain_step(&mut self, node: PipelineNode<Complex<f32>>) {
        let node_enum: PipelineNodeEnum = PipelineNodeEnum::Scalar(node);
        self.node_pool.push(node_enum);
    }

    pub fn add_frequency_domain_step(&mut self, node: PipelineNode<Vec<Complex<f32>>>) {
        let node_enum: PipelineNodeEnum = PipelineNodeEnum::Vector(node);
        self.node_pool.push(node_enum);
    }

    fn generate_threads(&mut self) -> Result<Vec<PipelineThread<ThreadType>>, ()> {
        let mut thread_constructors: Vec<PipelineThread<ThreadType>> = Vec::new();
        match self.node_pool.len() {
            Empty => Result::Err(()),
            1 => {},
            _ => {
                for (index, node) in self.node_pool.iter().skip(1).enumerate() {
                    let mut previous: &PipelineNodeEnum = self.node_pool.get(index - 1).unwrap();
                    let mut adapter: Option<Box<dyn PipelineNodeGeneric>> = self.welder.weld(&mut previous, &mut node);

                    match adapter {
                        None => {},
                        Some(node) => self.node_pool.insert(index, node)
                    }
                }
            }
        };

    }

    pub fn run(&mut self) {}

    pub fn pause(&mut self) {}

    pub fn resume(&mut self) {}

    pub fn end(&mut self) {}
}