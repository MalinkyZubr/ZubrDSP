use std::thread;
use std::time::Duration;
use std::sync::{Mutex, Arc, mpsc};
use async_std::task;

use super::node_enum::PipelineNodeEnum;


#[derive(Clone)]
pub enum PipelineThreadState {
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

impl<T: Clone + Send> PipelineThread<T> {
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
    pub fn call(&mut self) {
        let state = self.state.lock().unwrap();
        match *state {
            PipelineThreadState::RUNNING => self.node.lock().unwrap().call(),
            _ => thread::sleep(Duration::from_millis(500))
        };
    }
}