use std::fmt::Debug;
use std::thread::{self, sleep, JoinHandle};
use std::sync::{Mutex, Arc, RwLock, MutexGuard};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::Instant;
use super::pipeline_step::{PipelineStep, PipelineNode, CallableNode, PipelineStepResult};
use super::pipeline_traits::{HasID, Sharable};


pub struct PipelineThread {
    runflag: Arc<AtomicBool>,
    kill_flag: Arc<AtomicBool>,
    execution_time: Arc<AtomicU64>,
    pipeline_step_thread: Option<JoinHandle<()>>,
    notify_channel: (mpsc::SyncSender<()>, Option<mpsc::Receiver<()>>),
    return_code: Arc<RwLock<PipelineStepResult>>,
    pub id: String
}

impl PipelineThread {
    pub fn new<I: Sharable, O: Sharable>
    (step: impl PipelineStep<I, O> + 'static, node: impl CallableNode<I, O> + 'static + HasID) -> PipelineThread { // requires node to be borrowed as static?
        let execution_time = Arc::new(AtomicU64::new(0));
        let shared_state = Arc::new(AtomicBool::new(false));
        let kill_flag = Arc::new(AtomicBool::new(false));
        let notify_channel = mpsc::sync_channel(1);

        let mut thread = PipelineThread {
            runflag: shared_state,
            kill_flag,
            execution_time,
            pipeline_step_thread: None,
            notify_channel: (notify_channel.0, Some(notify_channel.1)),
            return_code: Arc::new(RwLock::new(PipelineStepResult::Success)),
            id: String::from("NoID"),
        };
        
        thread.instantiate_thread(step, node);
        
        return thread;
    }

    fn instantiate_thread<I: Sharable, O: Sharable>
    (&mut self, mut step: impl PipelineStep<I, O> + 'static, mut node: impl CallableNode<I, O> + 'static + HasID) {
            //let state_copy = Arc::new(self.runflag
            let runflag_clone = self.runflag.clone();
            let kill_flag_clone = self.kill_flag.clone();
            let execution_clone = self.execution_time.clone();
            let receiver = self.notify_channel.1.take().unwrap();
            let return_code_clone = self.return_code.clone();
            self.id = node.get_id();

            self.pipeline_step_thread = Some(
                thread::spawn(move || {
                    let mut error_count = 0;
                    while !kill_flag_clone.load(Ordering::Acquire) && error_count < 10 {
                        let _ = receiver.recv_timeout(std::time::Duration::from_millis(1000));

                        while runflag_clone.load(Ordering::Acquire) && error_count < 10 { // more thoughtful error handling later
                            let start_time = Instant::now();
                            let return_code_received = node.call(&mut step);

                            Self::increment_disconnect_counter(&return_code_received, &mut error_count);
                            *return_code_clone.write().unwrap() = return_code_received;

                            let execution_time = start_time.elapsed().as_millis() as u64;
                            execution_clone.store(execution_time, Ordering::SeqCst);
                        }
                    }
                })
            );
    }
    
    fn increment_disconnect_counter(result: &PipelineStepResult, disconnect_count: &mut u64) {
        match result {
            PipelineStepResult::Success => *disconnect_count = 0,
            _ => *disconnect_count += 1
        }
    }

    pub fn start(&mut self) {
        self.runflag.store(true, Ordering::Release);
        _ = self.notify_channel.0.try_send(());
        sleep(std::time::Duration::from_millis(10));
        //dbg!("STARTED");
    }

    pub fn stop(&mut self) {
        self.runflag.store(false, Ordering::Release);
        _ = self.notify_channel.0.try_send(());
        sleep(std::time::Duration::from_millis(10));
    }

    pub fn kill(&mut self) {
        match self.pipeline_step_thread.take() {
            Some(step_thread) => {
                self.runflag.store(false, Ordering::Release);
                self.kill_flag.store(true, Ordering::Release);
                
                sleep(std::time::Duration::from_millis(10));

                _ = self.notify_channel.0.try_send(());

                //println!("killing thread {}", self.id);
                _ = step_thread.join();
                //println!("thread {} killed", self.id);
            },
            None => {
                panic!("no thread is running!");
            }
        }
    }
}