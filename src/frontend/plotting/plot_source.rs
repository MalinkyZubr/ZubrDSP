use std::sync::Arc;
use crossbeam::queue::ArrayQueue;
use crate::pipeline::api::*;

pub struct PlotSource<T: Sharable> {
    input_queue: Arc<ArrayQueue<T>>,
}