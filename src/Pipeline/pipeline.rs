use num::complex::Complex;
use std::collections::{VecDeque};
use std::thread;
use std::time::Duration;
use std::sync::{Mutex, Arc, mpsc};
use async_std::task;

use super::buffer::{ScalarToVectorAdapter, VectorToScalarAdapter};
use super::node::prototype::{PipelineNode, PipelineNodeGeneric, PipelineStep};
use super::node::messages::{Source, Sink, ReceiverWrapper, SenderWrapper, create_node_connection};


pub enum PipelineNodeEnum<T> {
    Scalar(PipelineNode<T>),
    Vector(PipelineNode<T>),
    ScalarVectorAdapter(ScalarToVectorAdapter<T>),
    VectorScalarAdapter(VectorToScalarAdapter<T>)
}

impl<T> PipelineNodeEnum<T> {
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

#[derive(Clone)]
enum PipelineThreadState {
    RUNNING,
    STOPPED,
    ERROR(String),
    KILLED
}

pub struct PipelineThreadFriend {
    message_receiver: Option<mpsc::Receiver<PipelineThreadState>>,
    message_sender: Option<mpsc::Sender<PipelineThreadState>>,
}

pub struct ThreadTapManager {
    order_receive_task: Option<task::JoinHandle<()>>,
    diagnostic_send_task: Option<task::JoinHandle<()>>,
    // snapshot_send_task: 
    message_receiver: Option<mpsc::Receiver<PipelineThreadState>>,
    message_sender: Option<mpsc::Sender<PipelineThreadState>>,
}

impl ThreadTapManager {
    pub fn new(message_receiver: Option<mpsc::Receiver<PipelineThreadState>>, message_sender: Option<mpsc::Sender<PipelineThreadState>>) -> Self {
        Self {
            order_receive_task: None,
            diagnostic_send_task: None,
            message_receiver,
            message_sender
        }
    }

    async fn send_diagnostic(&mut self) {

    }

    // async fn 
    // fn start_taps(&mut self) -> Result<(), String> {
    //     match (self.message_receiver, self.message_sender) {
    //         (Some(sender), Some(receiver)) => {},
    //         (_, _) => {
    //             return Err("The receiver and sender were not set!".to_string());
    //         }
    //     };

    //     let mut message_receiver = self.message_receiver.unwrap();
    //     let mut message_sender = self.message_sender.unwrap();

    //     let mut state = self.state.clone();
    //     self.tap_thread = Some(thread::spawn(move || {
    //         while true {
    //             if let PipelineThreadState::KILLED = *state.lock().unwrap() {
    //                 break;
    //             }
    //             message_sender.send((*state.lock().unwrap()).clone());
    //         }
    //     }));

    //     return Ok(());
    // }
}

pub struct PipelineThread<T> {
    node: Arc<Mutex<PipelineNodeEnum<T>>>,
    state: Arc<Mutex<PipelineThreadState>>,
    tap_task_manager: ThreadTapManager
    // implement tap here for data so it can be viewed from outside
}

impl<T> PipelineThread<T> {
    pub fn new(node: PipelineNodeEnum<T>, message_receiver: Option<mpsc::Receiver<PipelineThreadState>>, message_sender: Option<mpsc::Sender<PipelineThreadState>>) -> PipelineThread<T> {
        PipelineThread {
            node: Arc::new(Mutex::new(node)), // requires node to be borrowed as static?
            state: Arc::new(Mutex::new(PipelineThreadState::STOPPED)),
            tap_task_manager: ThreadTapManager::new(message_receiver, message_sender)
        }
    }
    fn reset(&mut self) {

    }
    pub fn stop(&mut self) {
        let mut state = self.state.lock().unwrap();
        *state = PipelineThreadState::STOPPED;
    }
    pub fn resume(&mut self) {
        let mut state = self.state.lock().unwrap();
        match *state {
            PipelineThreadState::STOPPED => *state = PipelineThreadState::RUNNING,
            PipelineThreadState::ERROR(err) => {},
            _ => *state = PipelineThreadState::ERROR("Cannot resume non-stopped thread".to_string())
        };
    }
    pub fn kill(&mut self) {
        let mut state = self.state.lock().unwrap();
        *state = PipelineThreadState::KILLED;
    }
    pub fn run(&mut self) {
        self.reset();
        let mut state = self.state.lock().unwrap();
        *state = PipelineThreadState::RUNNING;
    }
    pub fn check_state(&self) -> &Arc<Mutex<PipelineThreadState>> {
        &self.state
    }
    fn call(&mut self) {
        let state = self.state.lock().unwrap();
        match *state {
            PipelineThreadState::RUNNING => self.node.lock().unwrap().call(),
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
    pub fn weld<T>(&self, src: &mut PipelineNodeEnum<T>, dst: &mut PipelineNodeEnum<T>) -> Option<PipelineNodeEnum<T>> {
        match (src, dst) {
            (PipelineNodeEnum::Scalar(src), 
            PipelineNodeEnum::Scalar(dst)) => {self.weld_scalar::<T>(src, dst); None},

            (PipelineNodeEnum::Vector(src), 
            PipelineNodeEnum::Vector(dst)) => {self.weld_vector::<T>(src, dst); None},

            (PipelineNodeEnum::Scalar(src), 
            PipelineNodeEnum::Vector(dst)) => Some(PipelineNodeEnum::ScalarVectorAdapter(self.weld_scalar_to_vector::<T>(src, dst))),

            (PipelineNodeEnum::Vector(src), 
            PipelineNodeEnum::Scalar(dst)) => Some(PipelineNodeEnum::VectorScalarAdapter(self.weld_vector_to_scalar::<T>(src, dst))),

            (_, _) => None
        }
    }

    fn weld_scalar<T>(&self, src: &mut PipelineNode<T>, dst: &mut PipelineNode<T>) {
        let (sender, receiver) = create_node_connection::<T>();
        src.set_output(Box::new(sender));
        dst.set_input(Box::new(receiver));
    }

    fn weld_vector<T>(&self, src: &mut PipelineNode<Vec<T>>, dst: &mut PipelineNode<Vec<T>>) {
        let (sender, receiver) = create_node_connection::<Vec<T>>();
        src.set_output(Box::new(sender));
        dst.set_input(Box::new(receiver));
    }

    fn weld_scalar_to_vector<T>(&self, src: &mut PipelineNode<T>, dst: &mut PipelineNode<Vec<T>>) -> ScalarToVectorAdapter<T> {
        let (scalar_sender, scalar_receiver) = create_node_connection::<T>();
        let (vector_sender, vector_receiver) = create_node_connection::<Vec<T>>();
        
        src.set_output(Box::new(scalar_sender));
        let adapter: ScalarToVectorAdapter = ScalarToVectorAdapter::new(scalar_receiver, vector_sender, self.buff_size);
        dst.set_input(Box::new(vector_receiver));

        return adapter;
    }

    fn weld_vector_to_scalar<T>(&self, src: &mut PipelineNode<Vec<T>>, dst: &mut PipelineNode<T>) -> VectorToScalarAdapter<T> {
        let (scalar_sender, scalar_receiver) = create_node_connection::<T>();
        let (vector_sender, vector_receiver) = create_node_connection::<Vec<T>>();
        
        src.set_output(Box::new(vector_sender));
        let adapter: VectorToScalarAdapter = VectorToScalarAdapter::new(vector_receiver, scalar_sender, self.buff_size);
        dst.set_input(Box::new(scalar_receiver));

        return adapter;
    }
}

pub struct Pipeline<T> {
    buff_size: usize,
    thread_pool: Vec<thread::JoinHandle<()>>,
    node_pool: VecDeque<PipelineNodeEnum<T>>,
    welder: Welder

}

impl<T> Pipeline<T> {
    pub fn new(buff_size: usize, source: PipelineNode<Vec<T>>, sink: PipelineNode<Vec<T>>) -> Pipeline<T> {
        let node_pool: VecDeque<PipelineNodeEnum<T>> = VecDeque::from([PipelineNodeEnum::Vector(source), PipelineNodeEnum::Vector(sink)]);
        Pipeline {
            buff_size,
            thread_pool: Vec::new(),
            node_pool,
            welder: Welder {buff_size}
        }
    }
    pub fn add_scalar_step(&mut self, step: Box<PipelineStep<T>>) {//node: PipelineNode<T>) {
        let node_enum: PipelineNodeEnum = PipelineNodeEnum::Scalar(PipelineNode::new(step));
        self.node_pool.insert(self.node_pool.len() - 1, node_enum);
    }

    pub fn add_vector_step(&mut self, step: Box<PipelineStep<Vec<T>>>) {
        let node_enum: PipelineNodeEnum = PipelineNodeEnum::Vector(PipelineNode::new(step));
        self.node_pool.insert(self.node_pool.len() - 1, node_enum);
    }

    fn threadify_nodes(&mut self, secondary: &mut VecDeque<PipelineNodeEnum<T>>) -> VecDeque<PipelineThread<T>> {
        let mut thread_containers: VecDeque<PipelineThread> = VecDeque::new();

        while secondary.len() > 0 {
            let node: PipelineNodeEnum = secondary.pop_front().unwrap();
            thread_containers.push_back(PipelineThread::new(node));
        };

        return thread_containers;
    }

    fn assemble_pipeline(&mut self) -> Result<VecDeque<PipelineThread<T>>, ()> {
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
            let adapter: Option<PipelineNodeEnum> = self.welder.weld::<T>(&mut previous, &mut current);

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

    fn run_pipeline(&mut self, mut thread_containers: VecDeque<PipelineThread<T>>) {
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