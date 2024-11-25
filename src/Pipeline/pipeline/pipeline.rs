use std::collections::VecDeque;
use num::Complex;
use std::thread;
use crate::Pipeline::node::prototype::{PipelineNode, PipelineStep};
use super::welder::Welder;
use super::node_enum::PipelineNodeEnum;
use super::thread::pipeline_thread::{PipelineThread, create_thread_and_tap};
use super::thread::thread_diagnostics::PipelineThreadState;
use super::thread::thread_friend::PipelineThreadFriend;



pub struct Pipeline<T: Clone + Send + 'static> {
    buff_size: usize,
    thread_pool: Vec<thread::JoinHandle<()>>,
    node_pool: VecDeque<PipelineNodeEnum<T>>,
    welder: Welder,
    thread_friends: Vec<PipelineThreadFriend>
}

impl<T: Clone + Send + 'static> Pipeline<T> {
    pub fn new(buff_size: usize, source: PipelineNode<Vec<T>>, sink: PipelineNode<Vec<T>>) -> Pipeline<T> {
        let node_pool: VecDeque<PipelineNodeEnum<T>> = VecDeque::from([PipelineNodeEnum::Vector(source), PipelineNodeEnum::Vector(sink)]);
        Pipeline {
            buff_size,
            thread_pool: Vec::new(),
            node_pool,
            welder: Welder {buff_size},
            thread_friends: Vec::new(),
        }
    }
    pub fn add_scalar_step(&mut self, step: Box<PipelineStep<T>>) {//node: PipelineNode<T>) {
        let node_enum: PipelineNodeEnum<T> = PipelineNodeEnum::Scalar(PipelineNode::new(step));
        self.node_pool.insert(self.node_pool.len() - 1, node_enum);
    }

    pub fn add_vector_step(&mut self, step: Box<PipelineStep<Vec<T>>>) {
        let node_enum: PipelineNodeEnum<T> = PipelineNodeEnum::Vector(PipelineNode::new(step));
        self.node_pool.insert(self.node_pool.len() - 1, node_enum);
    }

    fn threadify_nodes(&mut self, secondary: &mut VecDeque<PipelineNodeEnum<T>>) -> VecDeque<PipelineThread<T>> {
        let mut thread_containers: VecDeque<PipelineThread<T>> = VecDeque::new();

        while secondary.len() > 0 {
            let node: PipelineNodeEnum<T> = secondary.pop_front().unwrap();
            let (thread, friend) = create_thread_and_tap(node);
            thread_containers.push_back(thread);
            self.thread_friends.push(friend);
        };

        return thread_containers;
    }

    fn assemble_pipeline(&mut self) -> Result<VecDeque<PipelineThread<T>>, ()> {
        let mut thread_containers: VecDeque<PipelineThread<T>> = VecDeque::new();
        let mut secondary: VecDeque<PipelineNodeEnum<T>> = VecDeque::new();

        let length: usize = self.node_pool.len();

        if length == 0 {
            return Err(());
        }

        //thread_containers.push(PipelineThread::new(self.welder.weld(self.source, ).unwrap()));
        let mut previous = self.node_pool.pop_front().unwrap();

        while self.node_pool.len() > 0 {
            let mut current = self.node_pool.pop_front().unwrap();
            let adapter: Option<PipelineNodeEnum<T>> = self.welder.weld::<T>(&mut previous, &mut current);

            match adapter {
                None => {},
                Some(adpt) => {
                    let (thread, friend) = create_thread_and_tap(adpt);
                    thread_containers.push_back(thread);
                    self.thread_friends.push(friend);
                }
            }

            secondary.push_back(previous);
            previous = current;
        };
        
        thread_containers.extend(self.threadify_nodes(&mut secondary));

        return Ok(thread_containers);
    }

    fn run_pipeline(&mut self, mut thread_containers: VecDeque<PipelineThread<T>>) {
        while thread_containers.len() > 0 {
            let mut thread_container: PipelineThread<T> = thread_containers.pop_front().unwrap();

            let join_handle: thread::JoinHandle<()> = thread::spawn(move || {
                let state: &PipelineThreadState = &PipelineThreadState::STOPPED;
                while true {
                    if let PipelineThreadState::KILLED = state {
                        break;
                    }
                    else {
                        thread_container.call();
                    }
                    let state = &thread_container.check_state().lock().unwrap();
                }
            });
        }
    }

    pub fn run(&mut self) {}

    pub fn pause(&mut self) {}

    pub fn resume(&mut self) {}

    pub fn end(&mut self) {}
}


// two main types of pipelines supported. Serialization should take place in source and or sink components at the ends so that generics dont become too great an issue within pipeline
pub type DSPPipeline = Pipeline<Complex<f32>>;
pub type BytePipeline = Pipeline<u8>;