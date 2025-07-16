use std::fmt::Debug;
use std::sync::mpsc::Sender;
use crate::pipeline::pipeline_step::PipelineStep;

pub struct TapStep<T> {
    tap_sender: Sender<T>
}
impl<T> TapStep<T> {
    fn new(tap_sender: Sender<T>) -> Self {
        TapStep { tap_sender }
    }
}
impl<T: Send + Sync + Clone> PipelineStep<T, T> for TapStep<T> {
    fn run<'a>(&mut self, input: T) -> T {
        self.tap_sender.send(input.clone()).unwrap();

        return input;
    }
}