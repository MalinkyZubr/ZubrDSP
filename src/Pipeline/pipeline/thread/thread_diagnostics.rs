use std::sync::{Arc, RwLock};


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PipelineError {
    ResumeStoppedThread,
    ComputeError
}


#[derive(Clone, PartialEq, Debug, Copy)]
pub enum PipelineThreadState {
    RUNNING,
    STOPPED,
    ERROR(PipelineError),
    KILLED
}


pub struct BaseThreadDiagnostic {
    pub thread_state: Arc<RwLock<PipelineThreadState>>,
    pub execution_time: Arc<RwLock<f32>>
}


impl BaseThreadDiagnostic {
    pub fn new(thread_state: Arc<RwLock<PipelineThreadState>>, execution_time: Arc<RwLock<f32>>) -> BaseThreadDiagnostic {
        BaseThreadDiagnostic {
            thread_state,
            execution_time
        }
    }
}