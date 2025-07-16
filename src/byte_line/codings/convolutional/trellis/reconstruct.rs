use crate::pipeline::pipeline_step::PipelineStep;
use num::traits::{Pow};


// the output of the viterbi algorithm is a sequence of states. Needs further processing to extrac inputs
pub struct ConvolutionalReassembler {
    get_input_bitmask: u8,
}


impl ConvolutionalReassembler {
    pub fn new(input_size: u8) -> ConvolutionalReassembler {
        let mask = ((2 as u16).pow(input_size as u32) - 1) as u8;

        ConvolutionalReassembler {
            get_input_bitmask: mask,
        }
    }
    // assuming bytes are appended to msb
    pub fn compute_input_vector(&self, state_sequence: &[u8], input_sequence: &mut Vec<u8>) {
        for (index, state) in state_sequence.iter().enumerate() {
            input_sequence[index] = self.get_input_bitmask & state;
        }
    }
}

impl PipelineStep<Vec<u8>, Vec<u8>> for ConvolutionalReassembler {
    fn run(&mut self, input: Vec<u8>) -> Vec<u8> {
        let mut output: Vec<u8> = vec![0; input.len()];
        self.compute_input_vector(&input, &mut output);

        return output;
    }
}