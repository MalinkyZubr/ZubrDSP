// need a utility that allows to transition from different types. Should take in a vector of u8 for instance and convert to f32, or vice versa
use super::messages::{ReceiverWrapper, SenderWrapper, Source, Sink};
use super::prototype::PipelineNodeGeneric;
use std::collections::VecDeque;
use std::fmt::Debug;


// what is the goal: to have a standard format for efficient type conversion at different pipeline stages
pub struct TypeAdapterModule {
    in_receiver: ReceiverWrapper<T>,
    out_sender: SenderWrapper<Vec<T>>,
    copy_buffer: VecDeque<T>,
    // active_buffer: Vec<Complex<f32>>,
    buff_size: usize,
    counter: usize,
}