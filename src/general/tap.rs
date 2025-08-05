use std::fmt::Debug;
use std::sync::mpsc::Sender;
use crate::pipeline::api::*;

pub struct TapStep<T> {
    tap_sender: Sender<T>
}
impl<T> TapStep<T> {
    fn new(tap_sender: Sender<T>) -> Self {
        TapStep { tap_sender }
    }
}
impl<T: Sharable> PipelineStep<T, T> for TapStep<T> {
    fn run_SISO(&mut self, input: T) -> Result<ODFormat<T>, String> {
        self.tap_sender.send(input.clone()).unwrap(); Ok(ODFormat::Standard(input))
    }
}