use std::collections::VecDeque;
use std::fmt::Debug;
use num::Complex;
use std::thread;
use crate::pipeline::node::prototype::{PipelineNode, PipelineStep};
use super::welder::Welder;
use super::node_enum::PipelineNodeEnum;
use super::thread::pipeline_thread::{PipelineThread, create_thread_and_tap};
use super::thread::thread_diagnostics::PipelineThreadState;
use super::thread::thread_friend::PipelineThreadFriend;



pub struct Pipeline<T: Clone + Send + 'static + Debug> {
    buff_size: usize,
    thread_pool: Vec<thread::JoinHandle<()>>,
    node_pool: VecDeque<(PipelineNodeEnum<T>, String)>,
    welder: Welder,
    thread_friends: Vec<PipelineThreadFriend>
}

impl<T: Clone + Send + 'static + Debug> Pipeline<T> {
    pub fn new(buff_size: usize, source: PipelineNode<Vec<T>, Vec<T>>, sink: PipelineNode<Vec<T>, Vec<T>>) -> Pipeline<T> {
        let node_pool: VecDeque<(PipelineNodeEnum<T>, String)> = VecDeque::from([(PipelineNodeEnum::Vector(source), String::from("Source")), (PipelineNodeEnum::Vector(sink), String::from("Sink"))]);
        Pipeline {
            buff_size,
            thread_pool: Vec::new(),
            node_pool,
            welder: Welder {buff_size},
            thread_friends: Vec::new(),
        }
    }

    pub fn add_scalar_step(&mut self, step: Box<dyn PipelineStep<T, T>>, id: String) {//node: PipelineNode<T>) {
        let node_enum: PipelineNodeEnum<T> = PipelineNodeEnum::Scalar(PipelineNode::new(step));
        self.node_pool.insert(self.node_pool.len() - 1, (node_enum, id));
    }

    pub fn add_vector_step(&mut self, step: Box<dyn PipelineStep<Vec<T>, Vec<T>>>, id: String) {
        let node_enum: PipelineNodeEnum<T> = PipelineNodeEnum::Vector(PipelineNode::new(step));
        self.node_pool.insert(self.node_pool.len() - 1, (node_enum, id));
    }

    fn threadify_nodes(&mut self, secondary: &mut VecDeque<(PipelineNodeEnum<T>, String)>) -> VecDeque<PipelineThread<T>> {
        let mut thread_containers: VecDeque<PipelineThread<T>> = VecDeque::new();

        while secondary.len() > 0 {
            let (node, id): (PipelineNodeEnum<T>, String) = secondary.pop_front().unwrap();
            let (thread, friend) = create_thread_and_tap(node, id);
            thread_containers.push_back(thread);
            self.thread_friends.push(friend);
        };

        return thread_containers;
    }

    fn assemble_pipeline(&mut self) -> Result<VecDeque<PipelineThread<T>>, ()> {
        let mut thread_containers: VecDeque<PipelineThread<T>> = VecDeque::new();
        let mut secondary: VecDeque<(PipelineNodeEnum<T>, String)> = VecDeque::new();

        let length: usize = self.node_pool.len();

        if length == 0 {
            return Err(());
        }

        //thread_containers.push(PipelineThread::new(self.welder.weld(self.source, ).unwrap()));
        let mut previous: (PipelineNodeEnum<T>, String) = self.node_pool.pop_front().unwrap();

        while self.node_pool.len() > 0 {
            let mut current: (PipelineNodeEnum<T>, String) = self.node_pool.pop_front().unwrap();
            let adapter: Option<PipelineNodeEnum<T>> = self.welder.weld::<T>(&mut previous.0, &mut current.0);

            match adapter {
                None => {},
                Some(adpt) => {
                    let (thread, friend) = create_thread_and_tap(adpt, format!("{}-{}-Welder", &previous.1, &current.1));
                    thread_containers.push_back(thread);
                    self.thread_friends.push(friend);
                }
            }

            secondary.push_back(previous);
            previous = current;
        };

        secondary.push_back(previous);
        
        dbg!("SECONDARY LENGTH {}", secondary.len());
        thread_containers.extend(self.threadify_nodes(&mut secondary));

        return Ok(thread_containers);
    }

    fn run_pipeline(&mut self, mut thread_containers: VecDeque<PipelineThread<T>>) {
        while thread_containers.len() > 0 {
            let mut thread_container: PipelineThread<T> = thread_containers.pop_front().unwrap();

            let _join_handle: thread::JoinHandle<()> = thread::spawn(move || {
                while *thread_container.check_state().read().unwrap() != PipelineThreadState::KILLED{
                    thread_container.call();
                }
            });
        }
    }

    async fn send_watchdogs(&mut self) {

    }

    pub fn compose_threads(&mut self) {
        let pipeline = self.assemble_pipeline().unwrap();
        dbg!("PIPELINE LENGTH {}", pipeline.len());
        
        self.run_pipeline(pipeline);
    }

    pub async fn run(&mut self) {
        for friend in self.thread_friends.iter_mut() {
            friend.push_state(PipelineThreadState::RUNNING).await;
            dbg!("MASSIVE TURN ON!");
        }
    }

    pub async fn stop(&mut self) {
        for friend in self.thread_friends.iter_mut() {
            friend.push_state(PipelineThreadState::STOPPED).await;
        }
    }

    pub async fn end(&mut self) {
        for friend in self.thread_friends.iter_mut() {
            friend.push_state(PipelineThreadState::KILLED).await;
            dbg!("MASSIVE TURN OFF");
        }
    }
}


// two main types of pipelines supported. Serialization should take place in source and or sink components at the ends so that generics dont become too great an issue within pipeline
pub type DSPPipeline = Pipeline<Complex<f32>>;
pub type BytePipeline = Pipeline<u8>;