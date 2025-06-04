use std::{iter::Successors, sync::{Arc, Barrier, Mutex, RwLock, Condvar}, thread, time::Duration};
use crossbeam::deque::{Injector, Worker, Steal};


trait ComputeBounds = Send + Sync + Clone + 'static;

#[derive(Clone)]
pub struct ComputeModule<
        InputParam: ComputeBounds,
        PostcomputeParam: ComputeBounds,
        TaskReturn: ComputeBounds
    > {
    computation_task: fn(InputParam) -> TaskReturn,
    postcompute_task: fn(TaskReturn, &mut PostcomputeParam),
    postcompute_param: PostcomputeParam,
}

impl<
            InputParam: ComputeBounds,
            PostcomputeParam: ComputeBounds,
            TaskReturn: ComputeBounds
        >  
    ComputeModule<InputParam, PostcomputeParam, TaskReturn> {

    pub fn new(computation_task: fn(InputParam) -> TaskReturn, postcompute_task: fn(TaskReturn, &mut PostcomputeParam), postcompute_param: PostcomputeParam) -> Self {
        ComputeModule { computation_task, postcompute_task, postcompute_param }
    }

    pub fn executor(&mut self, task_queue: Arc<Injector<InputParam>>, running: Arc<RwLock<bool>>, is_empty: Arc<Condvar>) {
        let mut retry_flag = false;
        while *running.read().unwrap() || retry_flag {
            match task_queue.steal() {
                Steal::Empty => { 
                    retry_flag = false;
                    is_empty.notify_all();
                },
                Steal::Success(value) => {
                    let result = (self.computation_task)(value);
                    (self.postcompute_task)(result, &mut self.postcompute_param);
                    retry_flag = false;
                },
                Steal::Retry => retry_flag = true
            }
        }
    }
}

pub struct ParallelComputation<
        InputParam: ComputeBounds,
        PostcomputeParam: ComputeBounds,
        TaskReturn: ComputeBounds
    > {
    thread_pool: Vec<thread::JoinHandle<()>>,
    num_threads: usize,
    running: Arc<RwLock<bool>>,
    task_queue: Arc<Injector<InputParam>>,
    compute_module: ComputeModule<InputParam, PostcomputeParam, TaskReturn>,
    pub is_empty: Arc<Condvar>
}

impl<
            InputParam: ComputeBounds,
            PostcomputeParam: ComputeBounds,
            TaskReturn: ComputeBounds
        >  
    ParallelComputation<InputParam, PostcomputeParam, TaskReturn> {
    pub fn new(
                num_threads: usize, 
                compute_module: ComputeModule<InputParam, PostcomputeParam, TaskReturn>
            ) -> Self {
        ParallelComputation { 
            thread_pool: Vec::new(), 
            num_threads, 
            running: Arc::new(RwLock::new(false)), 
            task_queue: Arc::new(Injector::new()), 
            compute_module,
            is_empty: Arc::new(Condvar::new())
        }
    }

    pub fn add_task(&mut self, task: InputParam) {
        self.task_queue.push(task)
    }

    pub fn start(&mut self) {
        *self.running.write().unwrap() = true;
        let barrier = Arc::new(Barrier::new(self.num_threads));

        for _thread_num in 0..self.num_threads {
            let queue = self.task_queue.clone();
            let running = self.running.clone();
            let mut compute_module = self.compute_module.clone();
            let barrier_clone = barrier.clone();
            let condvar_clone = self.is_empty.clone();

            self.thread_pool.push(
                thread::spawn(move || {
                    barrier_clone.wait();
                    compute_module.executor(queue, running, condvar_clone);
                })
            )
        }
    }

    pub fn stop(mut self) {
        *self.running.write().unwrap() = false;

        for thread in self.thread_pool {
            let _ = thread.join();
        };
    }
}