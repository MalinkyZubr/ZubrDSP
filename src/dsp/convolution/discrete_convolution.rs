use std::mem;
use std::slice;
use crate::pipeline::pipeline_step::*;


pub struct DiscreteConvolution {
    reversed_impulse_response: Vec<f32>,
    input_size: usize,
    full_input: Vec<f32>,
    previous_input_size: usize
}

impl DiscreteConvolution {
    pub fn new(input_size: usize, mut impulse_response: Vec<f32>) -> Self {
        impulse_response.reverse();
        let ir_len = impulse_response.len();
        
        DiscreteConvolution {
            reversed_impulse_response: impulse_response,
            input_size,
            full_input: vec![0.0; input_size + ir_len - 1], // optimize this crap later, no cloning in the finished product!
            previous_input_size: ir_len - 1
        }
    }
    
    fn convolve_input(&mut self, mut input: Vec<f32>) -> Vec<f32> {
        let mut output = vec![0.0; self.input_size];
        let full_input_len = self.full_input.len();

        (&mut self.full_input[self.previous_input_size..]).swap_with_slice(&mut input.as_mut_slice()); // put the input into the full input efficiently

        for index in 0..self.input_size {
            let window = &mut self.full_input[index..index + self.reversed_impulse_response.len()];
            output[index] = window.iter().zip(self.reversed_impulse_response.iter()).map(|(a, b)| a * b).sum();
        }
        
        self.full_input.reverse();
        self.full_input[..self.previous_input_size].reverse();
        
        return output;
    }
}


impl PipelineStep<Vec<f32>, Vec<f32>> for DiscreteConvolution {
    fn run(&mut self, input: Vec<f32>) -> Vec<f32> {
        self.convolve_input(input)
    }
}