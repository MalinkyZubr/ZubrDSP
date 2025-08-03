use crate::dsp::fft::bit_reversal_optimized::*;
use crate::pipeline::api::*;
use num::Complex;
use crate::dsp::system_response::system_functions::{ImpulseResponse, TransferFunction};

pub struct OverlapAddChunker {
    input_size: usize,
    chunk_size: usize,
    padding_size: usize,
    num_chunks: usize,
}


impl OverlapAddChunker { // take in vec and turn it to vec of vec for overlap save system_response
    pub fn new(input_size: usize, impulse_response_size: usize, chunk_size: usize) -> Self {
        assert_eq!(input_size % chunk_size, 0);
        assert!(impulse_response_size <= chunk_size);
        assert!(chunk_size <= input_size);
        
        Self {
            input_size,
            chunk_size: chunk_size,
            padding_size: impulse_response_size - 1,
            num_chunks: input_size / chunk_size,
        }
    }
    fn generate_chunks(&self, mut data: Vec<f32>) -> Vec<Vec<f32>> {
        let mut output: Vec<Vec<f32>> = vec![Vec::with_capacity(self.chunk_size + self.padding_size); self.num_chunks];
        let chunked_input = data.chunks_mut(self.chunk_size);

        for (input_chunk, output_chunk) in chunked_input.zip(output.iter_mut()) {
            for index in 0..self.chunk_size {
                output_chunk[index] = input_chunk[index];
            }

            output_chunk.extend(vec![0.0; self.padding_size]);
        }
        
        output
    }
}

impl PipelineStep<Vec<f32>, Vec<Vec<f32>>> for OverlapAddChunker { // need some way for elegant chunk processing. This is something ill be doing often
    fn run(&mut self, input: ReceiveType<Vec<f32>>) -> Result<SendType<Vec<Vec<f32>>>, String> {
        match input {
            ReceiveType::Single(data) => Ok(SendType::NonInterleaved(self.generate_chunks(data))),
            _ => Err("chunker can only take single input vector".to_string())
        }
    }
}


pub struct OverlapAddCombiner {
    input_size: usize,
    chunk_size: usize,
    padding_size: usize,
    num_chunks: usize,
}
impl OverlapAddCombiner {
    pub fn new(input_size: usize, impulse_response_size: usize, chunk_size: usize) -> Self {
        assert_eq!(input_size % chunk_size, 0);
        assert!(impulse_response_size <= chunk_size);
        assert!(chunk_size <= input_size);

        Self {
            input_size,
            chunk_size: chunk_size,
            padding_size: impulse_response_size - 1,
            num_chunks: input_size / chunk_size,
        }
    }
    fn recombine_chunks(&self, mut data: Vec<Vec<f32>>) -> Vec<f32> {
        let mut output = Vec::with_capacity(self.num_chunks * self.chunk_size);
        
        output.extend_from_slice(data[0].as_slice());
        
        for (index, chunk) in data.iter_mut().enumerate().skip(1) {
            let (overlap, append) = chunk.split_at_mut(self.padding_size);
            
            let output_length = output.len();
            
            // overlap addition calculation
            for (output_value, overlap_value) in output.iter_mut().skip(output_length - self.padding_size).zip(overlap) {
                *output_value += *overlap_value;
            }
            output.extend_from_slice(append);
        };
        
        return output
    }
}
impl PipelineStep<Vec<Vec<f32>>, Vec<f32>> for OverlapAddCombiner { // need some way for elegant chunk processing. This is something ill be doing often
    fn run(&mut self, input: ReceiveType<Vec<Vec<f32>>>) -> Result<SendType<Vec<f32>>, String> {
        match input {
            ReceiveType::Single(data) => Ok(SendType::NonInterleaved(self.recombine_chunks(data))),
            _ => Err("chunker can only take single input vector".to_string())
        }
    }
}