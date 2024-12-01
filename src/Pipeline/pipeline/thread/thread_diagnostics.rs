use std::sync::{Arc, Mutex};


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PipelineError {
    ResumeStoppedThread,
}


#[derive(Clone, PartialEq, Debug, Copy)]
pub enum PipelineThreadState {
    RUNNING,
    STOPPED,
    ERROR(PipelineError),
    KILLED
}


pub struct BaseThreadDiagnostic {
    pub thread_state: Arc<Mutex<PipelineThreadState>>,
    pub execution_time: Arc<Mutex<f32>>
}


impl BaseThreadDiagnostic {
    pub fn new(thread_state: Arc<Mutex<PipelineThreadState>>, execution_time: Arc<Mutex<f32>>) -> BaseThreadDiagnostic {
        BaseThreadDiagnostic {
            thread_state,
            execution_time
        }
    }
}