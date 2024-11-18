use num::complex::Complex;
use std::collections::{VecDeque};
use std::thread;
use std::time::Duration;
use std::sync::{Mutex, Arc};

use super::buffer::{BufferType, ScalarToVectorAdapter, VectorToScalarAdapter};
use super::node::prototype::{PipelineNode, PipelineNodeGeneric, PipelineStep};
use super::node::messages::{Source, Sink, ReceiverWrapper, SenderWrapper, create_node_connection};


pub enum PipelineNodeEnum {
    Scalar(PipelineNode<Complex<f32>>),
    Vector(PipelineNode<Vec<Complex<f32>>>),
    ScalarVectorAdapter(ScalarToVectorAdapter),
    VectorScalarAdapter(VectorToScalarAdapter)
}

impl PipelineNodeEnum {
    // pub fn get_generalized(self) -> Arc<Mutex<dyn PipelineNodeGeneric + Send>> {
    //     match self {
    //         PipelineNodeEnum::Scalar(node) => Arc::new(Mutex::new(node)),
    //         PipelineNodeEnum::Vector(node) => Arc::new(Mutex::new(node)),
    //         PipelineNodeEnum::ScalarVectorAdapter(node) => Arc::new(Mutex::new(node)),
    //         PipelineNodeEnum::VectorScalarAdapter(node) => Arc::new(Mutex::new(node))
    //     }
    // }
    pub fn call(&mut self) {
        match self {
            PipelineNodeEnum::Scalar(node) => node.call(),
            PipelineNodeEnum::Vector(node) => node.call(),
            PipelineNodeEnum::ScalarVectorAdapter(node) => node.call(),
            PipelineNodeEnum::VectorScalarAdapter(node) => node.call()
        }
    }
}


enum PipelineThreadState {
    RUNNING,
    STOPPED,
    ERROR(String),
    KILLED
}

pub struct PipelineThread {
    node: PipelineNodeEnum,
    state: PipelineThreadState
}

impl PipelineThread {
    pub fn new(node: PipelineNodeEnum) -> PipelineThread {
        PipelineThread {
            node, // requires node to be borrowed as static?
            state: PipelineThreadState::STOPPED
        }
    }
    fn reset(&mut self) {

    }
    pub fn stop(&mut self) {
        self.state = PipelineThreadState::STOPPED;
    }
    pub fn resume(&mut self) {
        match &self.state {
            PipelineThreadState::STOPPED => self.state = PipelineThreadState::RUNNING,
            PipelineThreadState::ERROR(err) => {},
            _ => self.state = PipelineThreadState::ERROR("Cannot resume non-stopped thread".to_string())
        };
    }
    pub fn kill(&mut self) {
        self.state = PipelineThreadState::KILLED;
    }
    pub fn run(&mut self) {
        self.reset();
        self.state = PipelineThreadState::RUNNING;
    }
    pub fn check_state(&self) -> &PipelineThreadState {
        &self.state
    }
    fn call(&mut self) {
        match self.state {
            PipelineThreadState::RUNNING => self.node.call(),
            _ => thread::sleep(Duration::from_millis(500))
        };
    }
}

struct Welder {
    buff_size: usize
}

impl Welder {
    pub fn new(buff_size: usize) -> Welder {
        Welder {
            buff_size
        }
    }
    pub fn weld(&self, src: &mut PipelineNodeEnum, dst: &mut PipelineNodeEnum) -> Option<PipelineNodeEnum> {
        match (src, dst) {
            (PipelineNodeEnum::Scalar(src), 
            PipelineNodeEnum::Scalar(dst)) => {self.weld_scalar(src, dst); None},

            (PipelineNodeEnum::Vector(src), 
            PipelineNodeEnum::Vector(dst)) => {self.weld_vector(src, dst); None},

            (PipelineNodeEnum::Scalar(src), 
            PipelineNodeEnum::Vector(dst)) => Some(PipelineNodeEnum::ScalarVectorAdapter(self.weld_scalar_to_vector(src, dst))),

            (PipelineNodeEnum::Vector(src), 
            PipelineNodeEnum::Scalar(dst)) => Some(PipelineNodeEnum::VectorScalarAdapter(self.weld_vector_to_scalar(src, dst))),

            (_, _) => None
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

pub struct Pipeline {
    buff_size: usize,
    thread_pool: Vec<thread::JoinHandle<()>>,
    node_pool: VecDeque<PipelineNodeEnum>,
    welder: Welder

}

impl Pipeline {
    pub fn new(buff_size: usize, source: PipelineNode<Vec<Complex<f32>>>, sink: PipelineNode<Vec<Complex<f32>>>) -> Pipeline {
        let node_pool: VecDeque<PipelineNodeEnum> = VecDeque::from([PipelineNodeEnum::Vector(source), PipelineNodeEnum::Vector(sink)]);
        Pipeline {
            buff_size,
            thread_pool: Vec::new(),
            node_pool,
            welder: Welder {buff_size}
        }
    }
    pub fn add_scalar_step(&mut self, step: Box<PipelineStep<Complex<f32>>>) {//node: PipelineNode<Complex<f32>>) {
        let node_enum: PipelineNodeEnum = PipelineNodeEnum::Scalar(PipelineNode::new(step));
        self.node_pool.insert(self.node_pool.len() - 1, node_enum);
    }

    pub fn add_vector_step(&mut self, step: Box<PipelineStep<Vec<Complex<f32>>>>) {
        let node_enum: PipelineNodeEnum = PipelineNodeEnum::Vector(PipelineNode::new(step));
        self.node_pool.insert(self.node_pool.len() - 1, node_enum);
    }

    fn threadify_nodes(&mut self, secondary: &mut VecDeque<PipelineNodeEnum>) -> VecDeque<PipelineThread> {
        let mut thread_containers: VecDeque<PipelineThread> = VecDeque::new();

        while secondary.len() > 0 {
            let node: PipelineNodeEnum = secondary.pop_front().unwrap();
            thread_containers.push_back(PipelineThread::new(node));
        };

        return thread_containers;
    }

    fn assemble_pipeline(&mut self) -> Result<VecDeque<PipelineThread>, ()> {
        let mut thread_containers: VecDeque<PipelineThread> = VecDeque::new();
        let mut secondary: VecDeque<PipelineNodeEnum> = VecDeque::new();

        let length: usize = self.node_pool.len();

        if length == 0 {
            return Err(());
        }

        //thread_containers.push(PipelineThread::new(self.welder.weld(self.source, ).unwrap()));
        let mut previous = self.node_pool.pop_front().unwrap();

        while self.node_pool.len() > 0 {
            let mut current = self.node_pool.pop_front().unwrap();
            let adapter: Option<PipelineNodeEnum> = self.welder.weld(&mut previous, &mut current);

            match adapter {
                None => {},
                Some(adpt) => {
                    thread_containers.push_back(PipelineThread::new(adpt))
                }
            }

            secondary.push_back(previous);
            previous = current;
        };
        
        thread_containers.extend(self.threadify_nodes(&mut secondary));

        return Ok(thread_containers);
    }

    fn run_pipeline(&mut self, mut thread_containers: VecDeque<PipelineThread>) {
        while thread_containers.len() > 0 {
            let mut thread_container: PipelineThread = thread_containers.pop_front().unwrap();

            let join_handle: thread::JoinHandle<()> = thread::spawn(move || {
                let mut state: &PipelineThreadState = &PipelineThreadState::STOPPED;
                while true {
                    if let PipelineThreadState::KILLED = state {
                        break;
                    }
                    else {
                        thread_container.call();
                    }
                    state = thread_container.check_state();
                }
            });
        }
    }

    pub fn run(&mut self) {}

    pub fn pause(&mut self) {}

    pub fn resume(&mut self) {}

    pub fn end(&mut self) {}
}