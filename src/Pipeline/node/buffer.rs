use super::messages::{ReceiverWrapper, SenderWrapper, Source, Sink};
use super::prototype::PipelineNodeGeneric;
use std::collections::VecDeque;
use std::fmt::Debug;


pub struct ScalarToVectorAdapter<T: 'static + Send + Debug> {
    in_receiver: ReceiverWrapper<T>,
    out_sender: SenderWrapper<Vec<T>>,
    copy_buffer: VecDeque<T>,
    // active_buffer: Vec<Complex<f32>>,
    buff_size: usize,
    counter: usize,
}


pub struct VectorToScalarAdapter<T: 'static + Send + Debug> {
    in_receiver: ReceiverWrapper<Vec<T>>,
    out_sender: SenderWrapper<T>,
    buff_size: usize,
}


impl<T: Send + Clone + 'static + Debug> PipelineNodeGeneric for ScalarToVectorAdapter<T> {
    fn call(&mut self) {
        let in_data: T = self.in_receiver.recv().unwrap();
        self.copy_buffer.push_back(in_data); // dereference for copy
        
        if self.copy_buffer.len() == self.buff_size {
            match self.out_sender.send(Vec::from(self.copy_buffer.clone())) { // this is inefficient. Later on have a second field with pre-allocated space ready
                // that can store this while consumed by the next step so that it doesnt go out of scope. Pre-allocated cloning
                Ok(vec) => {}
                Err(msg) => {}
            }
        }
    }
}


impl<T: 'static + Send + Debug> ScalarToVectorAdapter<T> {
    pub fn new(in_receiver: ReceiverWrapper<T>, out_sender: SenderWrapper<Vec<T>>,
        // active_buffer: Vec<Complex<f32>>,
        buff_size: usize) -> ScalarToVectorAdapter<T> {
            ScalarToVectorAdapter {
                in_receiver,
                out_sender,
                copy_buffer: VecDeque::with_capacity(buff_size),
                buff_size,
                counter: 0
            }
    }
}


impl<T: Send + 'static + Debug> PipelineNodeGeneric for VectorToScalarAdapter<T> {
    fn call(&mut self) {        
        let mut in_data: VecDeque<T> = VecDeque::from(self.in_receiver.recv().unwrap());
        while in_data.len() > 0 {  
            dbg!("{}", &in_data.len()); 
            match self.out_sender.send(in_data.pop_front().unwrap()) {
                Ok(()) => {}
                Err(msg) => {}
            }
        }
    }
}


impl<T: 'static + Send + Debug> VectorToScalarAdapter<T> {
    pub fn new(in_receiver: ReceiverWrapper<Vec<T>>, out_sender: SenderWrapper<T>, buff_size: usize) -> VectorToScalarAdapter<T> {
        VectorToScalarAdapter {
            in_receiver: in_receiver,
            out_sender: out_sender,
            buff_size: buff_size
        }
    }
}
