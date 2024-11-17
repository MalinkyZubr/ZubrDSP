use std::sync::mpsc::{Receiver};
use num::complex::Complex;
use crate::Pipeline::node::messages::{ReceiverWrapper, SenderWrapper, Source, Sink};
use crate::Pipeline::node::prototype::PipelineNodeGeneric
pub trait BufferType {}

impl BufferType for Complex<f32> {}
impl BufferType for Vec<Complex<f32>> {} // make the size runtime based


// Each step is responsible for preparing its data for the next


pub struct ScalarToVectorAdapter {
    in_receiver: ReceiverWrapper<Complex<f32>>,
    out_sender: SenderWrapper<Vec<Complex<f32>>>,
    copy_buffer: Vec<Complex<f32>>,
    // active_buffer: Vec<Complex<f32>>,
    buff_size: usize,
    counter: usize,
}


pub struct VectorToScalarAdapter {
    in_receiver: ReceiverWrapper<Vec<Complex<f32>>>,
    out_sender: SenderWrapper<Complex<f32>>,
    buff_size: usize,
}


impl PipelineNodeGeneric for ScalarToVectorAdapter {
    fn call(&mut self) {
        let in_data: Complex<f32> = self.in_receiver.recv().unwrap();
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


impl ScalarToVectorAdapter {
    pub fn new(in_receiver: ReceiverWrapper<Complex<f32>>, out_sender: SenderWrapper<Vec<Complex<f32>>>,
        // active_buffer: Vec<Complex<f32>>,
        buff_size: usize) -> ScalarToVectorAdapter {
            ScalarToVectorAdapter {
                in_receiver: in_receiver,
                out_sender: out_sender,
                copy_buffer: Vec::new(),
                buff_size: buff_size,
                counter: 0
            }
    }
}


impl PipelineNodeGeneric for VectorToScalarAdapter {
    fn call(&mut self) {        
        let in_data: Vec<Complex<f32>> = self.in_receiver.recv().unwrap();
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


impl VectorToScalarAdapter {
    pub fn new(in_receiver: ReceiverWrapper<Vec<Complex<f32>>>, out_sender: SenderWrapper<Complex<f32>>, buff_size: usize) -> VectorToScalarAdapter {
        VectorToScalarAdapter {
            in_receiver: in_receiver,
            out_sender: out_sender,
            buff_size: buff_size
        }
    }
}
