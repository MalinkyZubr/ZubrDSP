use std::fmt::Debug;
use std::sync::mpsc::Sender;
use crate::pipeline::prototype::PipelineStep;

pub struct TapStep<T> {
    tap_sender: Sender<T>
}
impl<T> TapStep<T> {
    fn new(tap_sender: Sender<T>) -> Self {
        TapStep { tap_sender }
    }
}
impl<T: Send + Sync + Clone> PipelineStep<T, T> for TapStep<T> {
    fn run<'a>(&mut self, input: Option<T>) -> T {
        let input = input.unwrap();
        self.tap_sender.send(input.clone()).unwrap();

        return input;
    }
}