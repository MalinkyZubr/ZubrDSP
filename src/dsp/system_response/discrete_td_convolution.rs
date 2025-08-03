use std::mem;
use std::slice;
use crate::pipeline::api::*;
use super::system_functions::ImpulseResponse;


pub struct DiscreteConvolution {
    impulse_response: ImpulseResponse,
    external_impulse_response: bool,
    input_size: usize,
    full_input: Vec<f32>,
    previous_input_size: usize,
}

impl DiscreteConvolution {
    pub fn new(input_size: usize, impulse_response_length: usize, impulse_response: Option<ImpulseResponse>) -> Self {
        match impulse_response {
            Some(impulse_response) => {
                DiscreteConvolution {
                    impulse_response,
                    external_impulse_response: false,
                    input_size,
                    full_input: vec![0.0; input_size + impulse_response_length - 1], // optimize this crap later, no cloning in the finished product!
                    previous_input_size: impulse_response_length - 1,
                }
            }
            None => {
                DiscreteConvolution {
                    impulse_response: ImpulseResponse::new_configured(vec![0.0; impulse_response_length]),
                    external_impulse_response: true,
                    input_size,
                    full_input: vec![0.0; input_size + impulse_response_length - 1], // optimize this crap later, no cloning in the finished product!
                    previous_input_size: impulse_response_length - 1,
                }
            }
        }
    }
    
    fn convolve_input(&mut self, mut input: Vec<f32>) -> Vec<f32> {
        let mut output = vec![0.0; self.input_size];
        let full_input_len = self.full_input.len();

        (&mut self.full_input[self.previous_input_size..]).swap_with_slice(&mut input.as_mut_slice()); // put the input into the full input efficiently

        for index in 0..self.input_size {
            let window = &mut self.full_input[index..index + self.impulse_response.len()];
            output[index] = window.iter().zip(self.impulse_response.reversed_impulse_response.iter()).map(|(a, b)| a * b).sum();
        }
        
        self.full_input.reverse();
        self.full_input[..self.previous_input_size].reverse();
        
        return output;
    }


}


impl PipelineStep<Vec<f32>, Vec<f32>> for DiscreteConvolution {
    fn run(&mut self, input: ReceiveType<Vec<f32>>) -> Result<SendType<Vec<f32>>, String> {
        match (input, self.external_impulse_response) {
            (ReceiveType::Single(value), true) => Ok(SendType::NonInterleaved(self.convolve_input(value))),
            (ReceiveType::Multi(mut value), false) => {
                let mut impulse_response = value.pop().unwrap();
                impulse_response.reverse();
                self.impulse_response.reversed_impulse_response = impulse_response;
                let input = value.pop().unwrap();
                Ok(SendType::NonInterleaved(self.convolve_input(input)))
            },
            (ReceiveType::Multi(value), true) => Err(String::from("Cannot have static transfer function for multiple inputs")),
            (ReceiveType::Single(value), false) => Err(String::from("Must have static transfer function for single input")),
            _ => Err(String::from("Dummy Input"))
        }
    }
}