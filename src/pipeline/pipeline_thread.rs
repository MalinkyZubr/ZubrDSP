use std::fmt::Debug;
use std::thread::{self, JoinHandle};
use std::sync::{Mutex, Arc, RwLock, MutexGuard};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::Instant;
use super::prototype::{PipelineStep, PipelineNode, CallableNode, Sharable, HasID};


pub struct PipelineThread {
    state: Arc<AtomicBool>,
    kill_flag: Arc<AtomicBool>,
    execution_time: Arc<AtomicU64>,
    pipeline_step_thread: Option<JoinHandle<()>>,
    notify_channel: (mpsc::Sender<()>, Option<mpsc::Receiver<()>>),
    pub id: String
}

impl PipelineThread {
    pub fn new<I: Sharable, O: Sharable>
    (step: impl PipelineStep<I, O> + 'static, node: impl CallableNode<I, O> + 'static + HasID) -> PipelineThread { // requires node to be borrowed as static?
        let execution_time = Arc::new(AtomicU64::new(0));
        let shared_state = Arc::new(AtomicBool::new(false));
        let kill_flag = Arc::new(AtomicBool::new(false));
        let notify_channel = mpsc::channel();

        let mut thread = PipelineThread {
            state: shared_state,
            kill_flag,
            execution_time,
            pipeline_step_thread: None,
            notify_channel: (notify_channel.0, Some(notify_channel.1)),
            id: String::from("NoID"),
        };
        
        thread.instantiate_thread(step, node);
        
        return thread;
    }
    
    fn thread_operation<I: Sharable, O: Sharable>
    (
        mut step: impl PipelineStep<I, O> + 'static, mut node: impl CallableNode<I, O> + 'static, 
        kill_flag: Arc<AtomicBool>, 
        execution_time_storage: Arc<AtomicU64>, 
        receiver: mpsc::Receiver<()>, 
        state: Arc<AtomicBool>) 
    {
        while !kill_flag.load(Ordering::Relaxed) {
            let _ = receiver.recv();;
            
            while state.load(Ordering::Relaxed) {
                let start_time = Instant::now();

                node.call(&mut step);

                let execution_time = start_time.elapsed().as_millis() as u64;
                execution_time_storage.store(execution_time, Ordering::Relaxed);
            }
        }
    }

    fn instantiate_thread<I: Sharable, O: Sharable>
    (&mut self, step: impl PipelineStep<I, O> + 'static, node: impl CallableNode<I, O> + 'static + HasID) {
            //let state_copy = Arc::new(self.state
            let state_clone = self.state.clone();
            let kill_flag_clone = self.kill_flag.clone();
            let execution_clone = self.execution_time.clone();
            let receiver = self.notify_channel.1.take().unwrap();
            self.id = node.get_id();

            self.pipeline_step_thread = Some(
                thread::spawn(move || {
                    Self::thread_operation(
                        step, 
                        node,
                        kill_flag_clone,
                        execution_clone,
                        receiver,
                        state_clone
                    );
                })
            );
    }

    pub fn start(&mut self) {
        self.state.store(true, Ordering::SeqCst);
        self.notify_channel.0.send(()).unwrap();
    }

    pub fn stop(&mut self) {
        self.state.store(false, Ordering::SeqCst);
    }

    pub fn kill(&mut self) {
        match self.pipeline_step_thread.take() {
            Some(step_thread) => {
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