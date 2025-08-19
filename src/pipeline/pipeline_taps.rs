use std::sync::Arc;
use crossbeam_queue::ArrayQueue;
use super::api::*;


pub enum PipelineTap<T: Sharable> {
    None,
    Tap(Arc<ArrayQueue<T>>)
}
impl<T: Sharable> PipelineTap<T> {
    pub fn push(&mut self, item: T) {
        match self {
            PipelineTap::None => (),
            PipelineTap::Tap(queue) => { queue.push(item); () }
        }
    }
}