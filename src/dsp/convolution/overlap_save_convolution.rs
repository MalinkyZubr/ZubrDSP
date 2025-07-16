use crate::dsp::fft::bit_reversal_optimized::*;
use crate::pipeline::pipeline_step::PipelineStep;
use crate::pipeline::valid_types::*;
use crate::pipeline::pipeline_traits::Sharable;
use num::Complex;


pub struct Chunker<T: ValidDSPNumerical + Sharable> {
    input_size: usize,
    impulse_response_size: usize,
    input_chunk_size: usize,
    previous_input_block: Vec<T>
}

impl<T: ValidDSPNumerical + Sharable> Chunker<T> { // take in vec and turn it to vec of vec for overlap save convolution
    pub fn new(input_size: usize, impulse_response_size: usize, input_chunk_size: usize) -> Self {
        assert_eq!(input_size % input_chunk_size, 0);
        assert!(impulse_response_size <= input_chunk_size);
        assert!(input_chunk_size <= input_size);
        
        Self {
            input_size,
            impulse_response_size,
            input_chunk_size,
            previous_input_block: Vec::new()
        }
    }
}
// 
// impl<T: ValidDSPNumerical + Sharable> PipelineStep<Vec<T>, Vec<Vec<T>>> for Chunker<T> {
//     fn run(&mut self, input: Vec<T>) -> Vec<Vec<T>> {
//         
//     }
// }
