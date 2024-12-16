use std::borrow::Borrow;
use std::fmt::Debug;
use std::thread::{self, JoinHandle, Thread};
use std::time::Duration;
use std::sync::{Mutex, Arc, RwLock};
use async_std::channel::Recv;
use async_std::{task::{self, Task}, channel, io::timeout};
use futures::future::join_all;
use std::time::Instant;

use crate::Pipeline::pipeline::node_enum::PipelineNodeEnum;
use super::thread_diagnostics::{PipelineError, PipelineThreadState, BaseThreadDiagnostic};
use super::thread_friend::PipelineThreadFriend;


pub struct ThreadTapManager {
    message_receiver: Arc<channel::Receiver<PipelineThreadState>>,
    message_sender: Arc<channel::Sender<BaseThreadDiagnostic>>,
    state: Arc<RwLock<PipelineThreadState>>,
    execution_time: Arc<RwLock<f32>>,
}

impl ThreadTapManager {
    pub fn new(message_receiver: channel::Receiver<PipelineThreadState>, message_sender: channel::Sender<BaseThreadDiagnostic>, state: Arc<RwLock<PipelineThreadState>>, execution_time: Arc<RwLock<f32>>) -> Self {
        Self {
            message_receiver: Arc::new(message_receiver),
            message_sender: Arc::new(message_sender),
            state,
            execution_time
        }
    }

    async fn send_diagnostic(state: Arc<RwLock<PipelineThreadState>>, execution_time: Arc<RwLock<f32>>, message_sender: Arc<channel::Sender<BaseThreadDiagnostic>>) {
        while {
            let current_state = {
                let state = state.read().unwrap(); // Lock the mutex briefly
                *state
            };
            dbg!("{}", &current_state);
            current_state != PipelineThreadState::KILLED
        } {
            dbg!("THREAD STATE: {}", state.clone().read());
            let result = timeout(Duration::from_millis(100), async {
                let result = message_sender.send(
                BaseThreadDiagnostic::new(
                    state.clone(), 
                    execution_time.clone())
                ).await;

                Ok(())
            }).await;
            async_std::task::sleep(Duration::from_millis(100)).await
        }
        dbg!("ENDED SEND DIAGNOSTIC");
    }

    async fn receive_orders(state: Arc<RwLock<PipelineThreadState>>, message_receiver: Arc<channel::Receiver<PipelineThreadState>>) {
        while {
            let current_state = {
                let state = state.read().unwrap(); // Lock the mutex briefly
                *state
            };
            current_state != PipelineThreadState::KILLED
        } {
            let received_state: Result<PipelineThreadState, channel::RecvError> = timeout(Duration::from_millis(1000), async {
                    Ok(message_receiver.recv().await)
                }
            ).await.unwrap_or(Err(channel::RecvError));

            match received_state {
                Ok(result) => { 
                    dbg!("RECEIVED STATE: {}", result.clone());
                    let mut state = state.write().unwrap();
                    *state = result;
                },
                Err(error) => {
                    dbg!("RECEIVE ERROR!: {}", error);
                }
            }
            async_std::task::sleep(Duration::from_millis(100)).await
        }

        dbg!("ENDED RECEIVE ORDERS");
    }

    // async fn 
    pub async fn start_taps(self) -> Result<(), String> {
        let tasks = vec![task::spawn(
            ThreadTapManager::receive_orders(
                self.state.clone(), 
                self.message_receiver.clone())
            ),
            task::spawn(
                ThreadTapManager::send_diagnostic(
                    self.state.clone(), 
                    self.execution_time.clone(), 
                    self.message_sender.clone()
                )
            )
        ];

        let result = join_all(tasks).await;

        return Ok(());
    }
}


pub struct PipelineThread<T: Send + Clone + 'static + Debug> {
    node: Arc<Mutex<PipelineNodeEnum<T>>>,
    state: Arc<RwLock<PipelineThreadState>>,
    execution_time: Arc<RwLock<f32>>,
    tap_task_manager: Option<JoinHandle<()>>

    // implement tap here for data so it can be viewed from outside
}

impl<T: Clone + Send + 'static + Debug> PipelineThread<T> {
    pub fn new(node: PipelineNodeEnum<T>, message_receiver: channel::Receiver<PipelineThreadState>, message_sender: channel::Sender<BaseThreadDiagnostic>) -> PipelineThread<T> { // requires node to be borrowed as static?
        let state_arc: Arc<RwLock<PipelineThreadState>> =  Arc::new(RwLock::new(PipelineThreadState::STOPPED));
        let time_arc: Arc<RwLock<f32>> = Arc::new(RwLock::new(0 as f32));

        let tap_manager = ThreadTapManager::new(message_receiver, message_sender, state_arc.clone(), time_arc.clone());

        PipelineThread {
            node: Arc::new(Mutex::new(node)), // requires node to be borrowed as static?
            state: state_arc.clone(),
            execution_time: time_arc.clone(),
            tap_task_manager: Some(thread::spawn(move || {
                async_std::task::block_on(tap_manager.start_taps()); // use tokio if want to have separate async runtime for thread
            })),
        }
    }
    pub fn check_state(&self) -> &Arc<RwLock<PipelineThreadState>> {
        &self.state
    }
    pub fn call(&mut self) {
        let state = self.state.read().unwrap();
        let now = Instant::now();
        match *state {
            PipelineThreadState::RUNNING => self.node.lock().unwrap().call(),
            _ => thread::sleep(Duration::from_millis(500))
        };
        let mut execution_time = self.execution_time.write().unwrap();
        *execution_time = now.elapsed().as_secs_f32();
    }
}


pub fn create_thread_and_tap<T: Clone + Send + 'static + Debug>(node: PipelineNodeEnum<T>) -> (PipelineThread<T>, PipelineThreadFriend) {
    let (in_tx, in_rx) = channel::bounded(1); // maybe an issue to have unbounded if backups?
    let (out_tx, out_rx) = channel::bounded(1);

    let thread: PipelineThread<T> = PipelineThread::new(node, in_rx, out_tx);
    let thread_friend = PipelineThreadFriend::new(out_rx, in_tx);

    (thread, thread_friend)
}