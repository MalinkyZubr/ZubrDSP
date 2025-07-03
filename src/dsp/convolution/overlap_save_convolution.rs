use crate::dsp::fft::bit_reversal_optimized::*;


pub struct Chunker {
    input_size: usize,
    impulse_response_size: usize,
    input_chunk_size: usize,
    previous_input_block: Vec<f32>
}

impl Chunker {
    pub fn new(input_size: usize, impulse_response_size: usize, input_chunk_size: usize) -> Self {
        assert_eq!(input_size % input_chunk_size, 0);
        assert!(impulse_response_size <= input_chunk_size);
        assert!(input_chunk_size <= input_size);
        
        Self {
            input_size,
            impulse_response_size,
            input_chunk_size,
            previous_input_block: vec![0.0; impulse_response_size - 1]
        }
    }
    
    fn 
}

