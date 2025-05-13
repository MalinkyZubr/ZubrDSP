// need a utility that allows to transition from different types. Should take in a vector of u8 for instance and convert to f32, or vice versa
use std::fmt::Debug;

use super::messages::{ReceiverWrapper, SenderWrapper, Source, Sink};
use super::prototype::PipelineNodeGeneric;


pub trait TypeAdapterModule {

}