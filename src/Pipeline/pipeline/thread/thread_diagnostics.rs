#[derive(Clone, Copy)]
pub enum PipelineError {
    ResumeStoppedThread,
}


#[derive(Clone)]
pub enum PipelineThreadState {
    RUNNING,
    STOPPED,
    ERROR(PipelineError),
    KILLED
}


pub struct BaseThreadDiagnostic {
    thread_state: PipelineThreadState,
    execution_time: f32
}