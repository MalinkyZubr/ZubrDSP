use std::sync::mpsc::{Receiver};
use num::complex::Complex;
use crate::Pipeline::node::messages::{ReceiverWrapper, SenderWrapper, Source, Sink};
use crate::Pipeline::node::prototype::PipelineNodeGeneric;


pub struct ScalarToVectorAdapter<T> {
    in_receiver: ReceiverWrapper<T>,
    out_sender: SenderWrapper<Vec<T>>,
    copy_buffer: Vec<T>,
    // active_buffer: Vec<Complex<f32>>,
    buff_size: usize,
    counter: usize,
}


pub struct VectorToScalarAdapter<T> {
    in_receiver: ReceiverWrapper<Vec<T>>,
    out_sender: SenderWrapper<T>,
    buff_size: usize,
}


impl<T> PipelineNodeGeneric for ScalarToVectorAdapter<T> {
    fn call(&mut self) {
        let in_data: T = self.in_receiver.recv().unwrap();
        self.copy_buffer[self.counter] = in_data; // dereference for copy
        
        if self.counter == self.buff_size - 1 {
            self.counter = 0;

            match self.out_sender.send(self.copy_buffer.clone()) { // this is inefficient. Later on have a second field with pre-allocated space ready
                // that can store this while consumed by the next step so that it doesnt go out of scope. Pre-allocated cloning
                Ok(vec) => {}
                Err(msg) => {}
            }
        }
        else {
            self.counter += 1;
        }
    }
}


impl<T> ScalarToVectorAdapter<T> {
    pub fn new(in_receiver: ReceiverWrapper<T>, out_sender: SenderWrapper<Vec<T>>,
        // active_buffer: Vec<Complex<f32>>,
        buff_size: usize) -> ScalarToVectorAdapter<T> {
            ScalarToVectorAdapter {
                in_receiver: in_receiver,
                out_sender: out_sender,
                copy_buffer: Vec::new(),
                buff_size: buff_size,
                counter: 0
            }
    }
}


impl<T> PipelineNodeGeneric for VectorToScalarAdapter<T> {
    fn call(&mut self) {        
        let in_data: Vec<T> = self.in_receiver.recv().unwrap();
        let mut index: usize = 0;

        while index < self.buff_size {   
            match self.out_sender.send(in_data[index]) {
                Ok(()) => {}
                Err(msg) => {}
            }

            index += 1;
        }
    }
}


impl<T> VectorToScalarAdapter<T> {
    pub fn new(in_receiver: ReceiverWrapper<Vec<T>>, out_sender: SenderWrapper<T>, buff_size: usize) -> VectorToScalarAdapter<T> {
        VectorToScalarAdapter {
            in_receiver: in_receiver,
            out_sender: out_sender,
            buff_size: buff_size
        }
    }
}
