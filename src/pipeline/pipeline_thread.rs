use std::fmt::Debug;
use std::thread::{self, sleep, JoinHandle};
use std::sync::{Mutex, Arc, RwLock, MutexGuard};
use std::sync::atomic::{AtomicU64, Ordering, AtomicU8};
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use strum::Display;
use super::pipeline_step::{PipelineStep, PipelineNode, PipelineStepResult};
use super::pipeline_traits::{HasID, Sharable};
use super::api::*;


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
        if self.max_compute_errors == 0 { false }
        else if self.compute_errors_received > self.max_compute_errors {
            log_message(format!("ThreadID: {} max allowed compute errors received", id), Level::Warn);
            true
        }
        else { false }
    }
    pub fn infrastructure_error_lim_check(&mut self, id: &String) -> bool {
        if self.max_infrastructure_errors == 0 { false }
        else if self.infrastructure_errors_received > self.max_infrastructure_errors {
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
    state_change_messenger: mpsc::Sender<ThreadStateSpace>,
    id: String
}
impl ThreadStateMachine {
    pub fn new(parameters: &PipelineParameters, id: String, messenger: mpsc::Sender<ThreadStateSpace>) -> Self {
        let error_counter = ThreadErrorCounter::new(parameters.max_infrastructure_errors, parameters.max_compute_errors);
        Self { error_counter, state: ThreadStateSpace::PAUSED, id, no_change_timer: parameters.unchanged_state_time, state_change_messenger: messenger }
    }
    fn state_transition<I: Sharable, O: Sharable>(&mut self, requested_state: ThreadStateSpace, mut previous_output: PipelineStepResult, step: &mut impl PipelineStep<I, O>) {
        if requested_state == ThreadStateSpace::KILLED {
            self.kill_request_handler(step);
        }
        else if self.error_count_increment(&mut previous_output, step) {
            return;
        } else {
            match requested_state {
                ThreadStateSpace::PAUSED => self.pause_request_handler(step),
                ThreadStateSpace::RUNNING => self.running_request_handler(step),
                _ => ()
            }
        }
    }
    fn error_count_increment<I: Sharable, O: Sharable>(&mut self, previous_output: &mut PipelineStepResult, step: &mut impl PipelineStep<I, O>) -> bool {
        match previous_output {
            PipelineStepResult::SendError => {
                self.error_counter.infrastructure_error();
                log_message(format!("ThreadID: {} send error received", &self.id), Level::Warn);
                if self.error_counter.infrastructure_error_lim_check(&self.id) { self.set_kill_state_upstream(step) }
                true
            },
            PipelineStepResult::RecvTimeoutError(err) => {
                self.error_counter.infrastructure_error();
                match err {
                    RecvTimeoutError::Timeout => log_message(format!("ThreadID: {} receive timeout received", &self.id), Level::Warn),
                    RecvTimeoutError::Disconnected => log_message(format!("ThreadID: {} receiver disconnected received", &self.id), Level::Warn)
                }
                if self.error_counter.infrastructure_error_lim_check(&self.id) { self.set_kill_state_upstream(step) }
                true
            }
            PipelineStepResult::ComputeError(message) => {
                self.error_counter.compute_error();
                log_message(format!("ThreadID: {} compute error {}, pausing", &self.id, message), Level::Warn);
                if self.error_counter.compute_error_lim_check(&self.id) { self.set_pause_state_upstream(step) };
                true
            }
            PipelineStepResult::Success => { self.error_counter.success(); false }
            PipelineStepResult::Carryover => false
        }
    }
    fn set_kill_state<I: Sharable, O: Sharable>(&mut self, step: &mut impl PipelineStep<I, O>) {
        self.state = ThreadStateSpace::KILLED;
        step.kill_behavior();
        log_message(format!("ThreadID: {} state set killed", &self.id), Level::Info);
    }
    fn set_kill_state_upstream<I: Sharable, O: Sharable>(&mut self, step: &mut impl PipelineStep<I, O>) {
        self.set_kill_state(step);
        log_message(format!("Thread ID: {}, sending kill upstream", &self.id), Level::Debug);
        match self.state_change_messenger.send(ThreadStateSpace::KILLED) {
            Err(error) => panic!("Critical error, cannot reach management thread!"),
            Ok(_) => log_message(format!("Thread ID: {}, sent kill upstream", &self.id), Level::Debug)
        }
    }
    fn set_pause_state<I: Sharable, O: Sharable>(&mut self, step: &mut impl PipelineStep<I, O>) {
        self.state = ThreadStateSpace::PAUSED;
        step.pause_behavior();
        log_message(format!("ThreadID: {} state set paused", &self.id), Level::Info);
    }
    fn set_pause_state_upstream<I: Sharable, O: Sharable>(&mut self, step: &mut impl PipelineStep<I, O>) {
        self.set_pause_state(step);
        log_message(format!("Thread ID: {}, sending pause upstream", &self.id), Level::Debug);
        match self.state_change_messenger.send(ThreadStateSpace::PAUSED) {
            Err(error) => panic!("Critical error, cannot reach management thread!"),
            Ok(_) => log_message(format!("Thread ID: {}, sent pause upstream", &self.id), Level::Debug)
        }
    }
    fn set_running_state<I: Sharable, O: Sharable>(&mut self, step: &mut impl PipelineStep<I, O>) {
        self.state = ThreadStateSpace::RUNNING;
        step.start_behavior();
        log_message(format!("ThreadID: {} state set running", &self.id), Level::Info);
    }
    fn kill_request_handler<I: Sharable, O: Sharable>(&mut self, step: &mut impl PipelineStep<I, O>) {
        if self.state != ThreadStateSpace::KILLED {
            self.set_kill_state(step);
            log_message(format!("ThreadID: {} set kill state, exiting", &self.id), Level::Info);
        }
        else { sleep(Duration::from_millis(self.no_change_timer)) }
    }
    fn pause_request_handler<I: Sharable, O: Sharable>(&mut self, step: &mut impl PipelineStep<I, O>) {
        match self.state {
            ThreadStateSpace::PAUSED => sleep(Duration::from_millis(self.no_change_timer)),
            ThreadStateSpace::RUNNING => self.set_pause_state(step),
            ThreadStateSpace::KILLED => {
                log_message(format!("ThreadID: {} is killed, cannot set to paused state", &self.id), Level::Warn);
                sleep(Duration::from_millis(self.no_change_timer))
            }
        }
    }
    fn running_request_handler<I: Sharable, O: Sharable>(&mut self, step: &mut impl PipelineStep<I, O>) {
        match self.state {
            ThreadStateSpace::RUNNING => (),
            ThreadStateSpace::PAUSED => self.set_running_state(step),
            ThreadStateSpace::KILLED => {
                log_message(format!("ThreadID: {} is killed, cannot set to running state", &self.id), Level::Warn);
                sleep(Duration::from_millis(self.no_change_timer))
            }
        }
    }
    fn call<I: Sharable, O: Sharable>(&mut self, requested_state: ThreadStateSpace, previous_result: PipelineStepResult, node: &mut PipelineNode<I, O>, step: &mut impl PipelineStep<I, O>) -> PipelineStepResult {
        self.state_transition(requested_state, previous_result, step);

        let result = match self.state {
            ThreadStateSpace::RUNNING => {
                log_message(format!("ThreadID: {} call start", &self.id), Level::Debug);
                let res = node.call(step);
                log_message(format!("ThreadID: {} call done", &self.id), Level::Debug);
                res
            },
            _ => PipelineStepResult::Carryover
        };
        result
    }
}


pub struct PipelineThread {
    pub requested_state: Arc<AtomicU8>,
    pub execution_time: Arc<AtomicU64>,
    pipeline_step_thread: Option<JoinHandle<()>>,
    pub return_code: Arc<RwLock<PipelineStepResult>>,
    state_sender: Option<mpsc::Sender<ThreadStateSpace>>,
    pub id: String
}

impl PipelineThread {
    pub fn new<I: Sharable, O: Sharable>
    (step: impl PipelineStep<I, O> + 'static, node: PipelineNode<I, O>, parameters: PipelineParameters, state: (Arc<AtomicU8>, mpsc::Sender<ThreadStateSpace>)) -> PipelineThread { // requires node to be borrowed as static?
        let execution_time = Arc::new(AtomicU64::new(0));

        let mut thread = PipelineThread {
            execution_time,
            pipeline_step_thread: None,
            return_code: Arc::new(RwLock::new(PipelineStepResult::Success)),
            id: String::from("NoID"),
            requested_state: state.0, // all threads start as paused initially
            state_sender: Some(state.1)
        };
        
        thread.instantiate_thread(step, node, parameters);
        
        return thread;
    }

    fn instantiate_thread<I: Sharable, O: Sharable>
    (&mut self, mut step: impl PipelineStep<I, O> + 'static, mut node: PipelineNode<I, O>, parameters: PipelineParameters) {
        let execution_clone = self.execution_time.clone();
        let return_code_clone = self.return_code.clone();
        let state_receiver = self.requested_state.clone();
        let state_sender = self.state_sender.take();

        let state_sender = match state_sender {
            None => panic!("Cannot start step thread without attaching the managerial thread broadcaster"),
            Some(sender) => sender
        };

        self.id = node.get_id();

        self.pipeline_step_thread = Some(thread::spawn(move || { // refactor this so it isnt nonsense
            let mut state_machine = ThreadStateMachine::new(&parameters, node.get_id(), state_sender);
            let mut previous_result = PipelineStepResult::Carryover;
            let kill_comp = ThreadStateSpace::KILLED;

            while state_machine.state != kill_comp {
                let requested_state: ThreadStateSpace = ThreadStateSpace::try_from(state_receiver.load(Ordering::Acquire)).unwrap();
                log_message(format!("ThreadID: {} requested state {}, current state {}", node.get_id(), requested_state, state_machine.state), Level::Info);
                let start_time = Instant::now();
                previous_result = state_machine.call(requested_state, previous_result, &mut node, &mut step);

                execution_clone.store(start_time.elapsed().as_micros() as u64, Ordering::Release);

                if previous_result != PipelineStepResult::Carryover {
                    let mut write_guard = return_code_clone.write().unwrap();
                    *write_guard = previous_result.clone();
                }
            }
            log_message(format!("ThreadID: {} state machine end of action loop", node.get_id()), Level::Info); })
        );
    }
    
    pub fn join(self) {
        match self.pipeline_step_thread {
            None => panic!("pipeline thread already joined or was never created"),
            Some(handle) => {
                handle.join().unwrap();
                log_message(format!("PipelineThread: Joined pipeline thread {}", self.id), Level::Info);
            },
        }
    }
}


pub trait CollectibleThread {
    fn call_thread(&mut self);
}


impl<I: Sharable, O: Sharable> CollectibleThread for PipelineThread<I, O> {
    fn call_thread(&mut self) {
        
    }
}