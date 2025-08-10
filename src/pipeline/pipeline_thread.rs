use std::fmt::Debug;
use std::thread::{self, sleep, JoinHandle};
use std::sync::{Mutex, Arc, RwLock, MutexGuard};
use std::sync::atomic::{AtomicU64, Ordering, AtomicU8};
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use crate::pipeline::api::PipelineStepResult::Carryover;
use super::pipeline_step::{PipelineStep, PipelineNode, CallableNode, PipelineStepResult};
use super::pipeline_traits::{HasID, Sharable};
use super::api::*;


#[repr(u8)]
#[derive(PartialEq, Debug, TryFromPrimitive)]
enum ThreadStateSpace {
    RUNNING = 0,
    PAUSED = 1,
    KILLED = 2
}


struct ThreadErrorCounter {
    max_infrastructure_errors: usize,
    infrastructure_errors_received: usize, // kill when reach limit
    max_compute_errors: usize,
    compute_errors_received: usize, // pause when reach limit
}
impl ThreadErrorCounter {
    pub fn new(max_infrastructure_errors: usize, max_compute_errors: usize) -> Self {
        Self { max_compute_errors, max_infrastructure_errors, infrastructure_errors_received: 0, compute_errors_received: 0 }
    }
    pub fn infrastructure_error(&mut self) {
        self.infrastructure_errors_received += 1;
    }
    pub fn compute_error(&mut self) {
        self.compute_errors_received += 1;
    }
    pub fn success(&mut self) {
        self.infrastructure_errors_received = 0;
        self.compute_errors_received = 0;
    }
    pub fn compute_error_lim_check(&mut self, id: &String) -> bool {
        if self.compute_errors_received > self.max_compute_errors {
            log_message(format!("ThreadID: {} max allowed compute errors received", id), Level::Warn);
            true
        }
        else { false }
    }
    pub fn infrastructure_error_lim_check(&mut self, id: &String) -> bool{
        if self.infrastructure_errors_received > self.max_infrastructure_errors {
            log_message(format!("ThreadID: {} max allowed infrastructure errors received", id), Level::Error);
            true
        }
        else { false }
    }
}

struct ThreadStateMachine {
    state: ThreadStateSpace,
    error_counter: ThreadErrorCounter,
    no_change_timer: u64,
    id: String
}
impl ThreadStateMachine {
    pub fn new(parameters: &PipelineParameters, id: String) -> Self {
        let error_counter = ThreadErrorCounter::new(parameters.max_infrastructure_errors, parameters.max_compute_errors);
        Self { error_counter, state: ThreadStateSpace::PAUSED, id, no_change_timer: parameters.unchanged_state_time }
    }
    fn state_transition(&mut self, requested_state: ThreadStateSpace, mut previous_output: PipelineStepResult) {
        if self.error_count_increment(&mut previous_output) {
            return;
        } else {
            match requested_state {
                ThreadStateSpace::KILLED => {
                    if self.state != ThreadStateSpace::KILLED {
                        self.state = ThreadStateSpace::KILLED;
                        log_message(format!("ThreadID: {} set kill state, exiting", &self.id), Level::Info);
                    }
                    else { sleep(Duration::from_millis(self.no_change_timer)) }
                },
                ThreadStateSpace::PAUSED => {
                    self.pause_request_handler();
                }
                ThreadStateSpace::RUNNING => {
                    self.running_request_handler();
                }
            }
        }
    }
    fn error_count_increment(&mut self, previous_output: &mut PipelineStepResult) -> bool {
        match previous_output {
            PipelineStepResult::SendError => {
                self.error_counter.infrastructure_error();
                log_message(format!("ThreadID: {} send error received", &self.id), Level::Warn);
                if self.error_counter.infrastructure_error_lim_check(&self.id) { self.set_kill_state() }
                true
            },
            PipelineStepResult::RecvTimeoutError(err) => {
                self.error_counter.infrastructure_error();
                match err {
                    RecvTimeoutError::Timeout => log_message(format!("ThreadID: {} receive timeout received", &self.id), Level::Warn),
                    RecvTimeoutError::Disconnected => log_message(format!("ThreadID: {} receiver disconnected received", &self.id), Level::Warn)
                }
                if self.error_counter.infrastructure_error_lim_check(&self.id) { self.set_kill_state() }
                true
            }
            PipelineStepResult::ComputeError(message) => {
                self.error_counter.compute_error();
                log_message(format!("ThreadID: {} compute error {}, pausing", &self.id, message), Level::Warn);
                if self.error_counter.compute_error_lim_check(&self.id) { self.set_pause_state() };
                true
            }
            PipelineStepResult::Success => { self.error_counter.success(); false }
            PipelineStepResult::Carryover => false
        }
    }
    fn set_kill_state(&mut self) {
        self.state = ThreadStateSpace::KILLED;
        log_message(format!("ThreadID: {} state set killed", &self.id), Level::Info);
    }
    fn set_pause_state(&mut self) {
        self.state = ThreadStateSpace::PAUSED;
        log_message(format!("ThreadID: {} state set paused", &self.id), Level::Info);
    }
    fn set_running_state(&mut self) {
        self.state = ThreadStateSpace::RUNNING;
        log_message(format!("ThreadID: {} state set running", &self.id), Level::Info);
    }
    fn pause_request_handler(&mut self) {
        match self.state {
            ThreadStateSpace::PAUSED => sleep(Duration::from_millis(self.no_change_timer)),
            ThreadStateSpace::RUNNING => self.set_pause_state(),
            ThreadStateSpace::KILLED => {
                log_message(format!("ThreadID: {} is killed, cannot set to paused state", &self.id), Level::Warn);
                sleep(Duration::from_millis(self.no_change_timer))
            }
        }
    }
    fn running_request_handler(&mut self) {
        match self.state {
            ThreadStateSpace::RUNNING => (),
            ThreadStateSpace::PAUSED => self.set_running_state(),
            ThreadStateSpace::KILLED => {
                log_message(format!("ThreadID: {} is killed, cannot set to running state", &self.id), Level::Warn);
                sleep(Duration::from_millis(self.no_change_timer))
            }
        }
    }
    fn call<I: Sharable, O: Sharable>(&mut self, requested_state: ThreadStateSpace, previous_result: PipelineStepResult, node: &mut impl CallableNode<I, O>, step: &mut impl PipelineStep<I, O>) -> PipelineStepResult {
        self.state_transition(requested_state, previous_result);

        match self.state {
            ThreadStateSpace::RUNNING => node.call(step),
            _ => PipelineStepResult::Carryover
        }
    }
}


pub struct PipelineThread {
    requested_state: Arc<AtomicU8>,
    execution_time: Arc<AtomicU64>,
    pipeline_step_thread: Option<JoinHandle<()>>,
    return_code: Arc<RwLock<PipelineStepResult>>,
    pub id: String
}

impl PipelineThread {
    pub fn new<I: Sharable, O: Sharable>
    (step: impl PipelineStep<I, O> + 'static, node: impl CallableNode<I, O> + 'static + HasID, parameters: PipelineParameters) -> PipelineThread { // requires node to be borrowed as static?
        let execution_time = Arc::new(AtomicU64::new(0));

        let mut thread = PipelineThread {
            execution_time,
            pipeline_step_thread: None,
            return_code: Arc::new(RwLock::new(PipelineStepResult::Success)),
            id: String::from("NoID"),
            requested_state: Arc::new(AtomicU8::new(1)) // all threads start as paused initially
        };
        
        thread.instantiate_thread(step, node, parameters);
        
        return thread;
    }

    fn instantiate_thread<I: Sharable, O: Sharable>
    (&mut self, mut step: impl PipelineStep<I, O> + 'static, mut node: impl CallableNode<I, O> + 'static + HasID, parameters: PipelineParameters) {
        let execution_clone = self.execution_time.clone();
        let return_code_clone = self.return_code.clone();
        let state_receiver = self.requested_state.clone();
        self.id = node.get_id();

        self.pipeline_step_thread = Some(
            thread::spawn(move || { // refactor this so it isnt nonsense
                let mut state_machine = ThreadStateMachine::new(&parameters, node.get_id());
                let mut previous_result = Carryover;

                while state_machine.state != ThreadStateSpace::KILLED {
                    let requested_state: ThreadStateSpace = ThreadStateSpace::try_from(state_receiver.load(Ordering::Acquire)).unwrap();
                    let start_time = Instant::now();
                    previous_result = state_machine.call(requested_state, previous_result, &mut node, &mut step);
                    execution_clone.store(start_time.elapsed().as_secs(), Ordering::Release);
                    
                    if previous_result != PipelineStepResult::Carryover {
                        let mut write_guard = return_code_clone.write().unwrap();
                        *write_guard = previous_result.clone();
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
        self.requested_state.store(0, Ordering::Release);
        sleep(std::time::Duration::from_millis(10));
        //dbg!("STARTED");
    }

    pub fn stop(&mut self) {
        self.requested_state.store(1, Ordering::Release);
        sleep(std::time::Duration::from_millis(10));
    }

    pub fn kill(&mut self) {
        match self.pipeline_step_thread.take() {
            Some(step_thread) => {
                self.requested_state.store(2, Ordering::Release);
                
                sleep(std::time::Duration::from_millis(10));

                log_message(format!("Stopping thread: {}", self.id), Level::Debug);
                _ = step_thread.join();
                log_message(format!("Thread killed: {}", self.id), Level::Debug);
            },
            None => {
                panic!("no thread is running!");
            }
        }
    }
}