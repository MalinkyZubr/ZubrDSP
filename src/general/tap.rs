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
    fn run(&mut self, input: ReceiveType<T>) -> Result<SendType<T>, String> {
        match input {
            ReceiveType::Single(value) => { self.tap_sender.send(value.clone()).unwrap(); Ok(SendType::NonInterleaved(value)) },
            _ => Err(String::from("Unexpected receive type."))
        }
    }
}