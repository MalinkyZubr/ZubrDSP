use std::fmt::Debug;
use std::thread::{self, sleep, JoinHandle};
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
    notify_channel: (mpsc::SyncSender<()>, Option<mpsc::Receiver<()>>),
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
        while !kill_flag.load(Ordering::Acquire) {
            let _ = receiver.recv_timeout(std::time::Duration::from_millis(1000));
            //dbg!("BULLSHIT!");

            while state.load(Ordering::Acquire) {
                //dbg!("I SHIT WHERE I EAT");
                let start_time = Instant::now();
                node.call(&mut step);

                let execution_time = start_time.elapsed().as_millis() as u64;
                execution_time_storage.store(execution_time, Ordering::SeqCst);
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
        self.state.store(true, Ordering::Release);
        _ = self.notify_channel.0.send(());
        sleep(std::time::Duration::from_millis(10));
        //dbg!("STARTED");
    }

    pub fn stop(&mut self) {
        self.state.store(false, Ordering::Release);
        _ = self.notify_channel.0.send(());
        sleep(std::time::Duration::from_millis(10));
    }

    pub fn kill(&mut self) {
        match self.pipeline_step_thread.take() {
            Some(step_thread) => {
                self.state.store(false, Ordering::Release);
                self.kill_flag.store(true, Ordering::Release);
                
                sleep(std::time::Duration::from_millis(10));

                _ = self.notify_channel.0.send(());

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