use std::thread;
use std::time::Duration;
use std::sync::{Mutex, Arc, mpsc};
use async_std::task;
use std::time::Instant;

use crate::Pipeline::pipeline::node_enum::PipelineNodeEnum;
use super::thread_diagnostics::{PipelineError, PipelineThreadState, BaseThreadDiagnostic};
use super::thread_friend::PipelineThreadFriend;


pub struct ThreadTapManager {
    order_receive_task: Option<task::JoinHandle<()>>,
    managerial_thread: Option<thread::JoinHandle<()>>,
    message_receiver: mpsc::Receiver<PipelineThreadState>,
    message_sender: mpsc::Sender<BaseThreadDiagnostic>,
    state: Arc<Mutex<PipelineThreadState>>,
    execution_time: Arc<Mutex<f32>>,
}

impl ThreadTapManager {
    pub fn new(message_receiver: mpsc::Receiver<PipelineThreadState>, message_sender: mpsc::Sender<BaseThreadDiagnostic>, state: Arc<Mutex<PipelineThreadState>>, execution_time: Arc<Mutex<f32>>) -> Self {
        Self {
            order_receive_task: None,
            managerial_thread: None,
            message_receiver,
            message_sender,
            state,
            execution_time
        }
    }

    async fn send_diagnostic(&mut self) {

    }

    // async fn 
    fn start_taps(&mut self) -> Result<(), String> {
        let mut message_receiver = &self.message_receiver;
        let mut message_sender = &self.message_sender;

        let mut state = self.state.clone();
        self.managerial_thread = Some(thread::spawn(move || {
            while true {
                if let PipelineThreadState::KILLED = *state.lock().unwrap() {
                    break;
                }
                
            }
        }));

        return Ok(());
    }
}

pub struct PipelineThread<T: Send + Clone + 'static> {
    node: Arc<Mutex<PipelineNodeEnum<T>>>,
    state: Arc<Mutex<PipelineThreadState>>,
    execution_time: Arc<Mutex<f32>>,
    tap_task_manager: ThreadTapManager
    // implement tap here for data so it can be viewed from outside
}

impl<T: Clone + Send + 'static> PipelineThread<T> {
    pub fn new(node: PipelineNodeEnum<T>, message_receiver: mpsc::Receiver<PipelineThreadState>, message_sender: mpsc::Sender<BaseThreadDiagnostic>) -> PipelineThread<T> { // requires node to be borrowed as static?
        let state_arc: Arc<Mutex<PipelineThreadState>> =  Arc::new(Mutex::new(PipelineThreadState::STOPPED));
        let time_arc: Arc<Mutex<f32>> = Arc::new(Mutex::new(0 as f32));

        PipelineThread {
            node: Arc::new(Mutex::new(node)), // requires node to be borrowed as static?
            state: state_arc.clone(),
            execution_time: time_arc.clone(),
            tap_task_manager: ThreadTapManager::new(message_receiver, message_sender, state_arc.clone(), time_arc.clone()),
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
            _ => *state = PipelineThreadState::ERROR(PipelineError::ResumeStoppedThread)
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
    pub fn call(&mut self) {
        let state = self.state.lock().unwrap();
        let now = Instant::now();
        match *state {
            PipelineThreadState::RUNNING => self.node.lock().unwrap().call(),
            _ => thread::sleep(Duration::from_millis(500))
        };
        let mut execution_time = self.execution_time.lock().unwrap();
        *execution_time = now.elapsed().as_secs_f32();
    }
}


pub fn create_thread_and_tap<T: Clone + Send + 'static>(node: PipelineNodeEnum<T>) -> (PipelineThread<T>, PipelineThreadFriend) {
    let (in_tx, in_rx) = mpsc::channel();
    let (out_tx, out_rx) = mpsc::channel();

    let thread: PipelineThread<T> = PipelineThread::new(node, in_rx, out_tx);
    let thread_friend = PipelineThreadFriend::new(out_rx, in_tx);

    (thread, thread_friend)
}