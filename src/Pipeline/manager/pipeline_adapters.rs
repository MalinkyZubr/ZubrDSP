use crate::Pipeline::node::prototype::{PipelineNodeGeneric};


pub struct ByteToDSPAdapter {
    in_receiver: ReceiverWrapper<Vec<u8>>,
    out_sender: SenderWrapper<Vec<Complex<f32>>>,
}


impl<T: SEnd + Clone + 'static + Debug> PipelineNodeGeneric for ByteToDSPAdapter {
    
}