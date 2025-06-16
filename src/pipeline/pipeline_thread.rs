use std::fmt::Debug;
use std::thread::{self, JoinHandle};
use std::sync::{Mutex, Arc, RwLock, MutexGuard};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use async_std::{task::{self}, channel, io::timeout};
use std::time::Instant;
use futures::task::SpawnExt;
use atomic_enum;
use super::prototype::{PipelineStep, PipelineNode};
use super::node_enum::{PipelineStepEnum};


pub struct PipelineThread {
    state: Arc<AtomicBool>,
    kill_flag: Arc<AtomicBool>,
    execution_time: Arc<AtomicU64>,
    pipeline_step_thread: Option<JoinHandle<()>>,
    pub id: String
}

impl PipelineThread {
    pub fn new(id: String) -> PipelineThread { // requires node to be borrowed as static?
        let execution_time = Arc::new(AtomicU64::new(0));
        let shared_state = Arc::new(AtomicBool::new(false));
        let kill_flag = Arc::new(AtomicBool::new(false));

        //let tap_manager = ThreadTapManager::new(message_receiver, message_sender, time_arc.clone());

        PipelineThread {
            state: shared_state,
            kill_flag,
            execution_time,
            // tap_task_manager: Some(thread::spawn(move || {
            //     async_std::task::block_on(tap_manager.start_taps()); // use tokio if want to have separate async runtime for thread
            // })),
            pipeline_step_thread: None,
            execution_barrier: Arc::new(Mutex::new(())),
            id
        }
    }

    pub fn instantiate_thread<I: Send + Clone + Debug + 'static, O: Send + Clone + Debug + 'static>
    (&mut self, mut step: impl PipelineStep<I, O> + 'static, mut node_wrapped: Arc<Mutex<PipelineNode<I, O>>>) {
            //let state_copy = Arc::new(self.state
            let state_clone = self.state.clone();
            let kill_flag_clone = self.kill_flag.clone();
            let mut execution_clone = self.execution_time.clone();

            self.pipeline_step_thread = Some(
                thread::spawn(move || {
                    while !kill_flag_clone.load(Ordering::Relaxed) {
                        {let mut node = node_wrapped.lock().unwrap();
                        while state_clone.load(Ordering::Relaxed) {
                            let start_time = Instant::now();

                            node.call(&mut step);

                            let execution_time = start_time.elapsed().as_millis() as u64;
                            execution_clone.store(execution_time, Ordering::Relaxed);
                        }}
                    }
                })
            );
    }

    pub fn start(&mut self) {
        self.state.store(true, Ordering::SeqCst);
    }

    pub fn stop(&mut self) {
        self.state.store(false, Ordering::SeqCst);
    }

    pub fn kill(&mut self) {
        match self.pipeline_step_thread.take() {
            Some(mut step_thread) => {
                self.state.store(false, Ordering::SeqCst);
                self.kill_flag.store(true, Ordering::SeqCst);
                step_thread.join().unwrap();
            },
            None => {
                panic!("no thread is running!");
            }
        }
    }
}