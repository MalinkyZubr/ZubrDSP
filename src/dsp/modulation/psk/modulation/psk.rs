use crate::pipeline::api::*;
use std::f32;


#[derive(Clone, PartialEq, Debug)]
pub enum BasisType {
    COSINE,
    SINE
}


#[derive(Clone, PartialEq, Debug)]
pub struct PSKModulator {
    samples_per_symbol: f32,
    basis_type: BasisType
}

impl PSKModulator {
    pub fn new(samples_per_symbol: f32, basis_type: BasisType) -> Self {
        PSKModulator {
            samples_per_symbol,
            basis_type
        }
    }

    fn execute_basis(&self, time_value: f32) -> f32 {
        match &self.basis_type {
            BasisType::COSINE => f32::cos(time_value),
            BasisType::SINE => f32::sin(time_value)
        }
    }

    fn generate_symbol(&self, phase_offset: &f32, output_buffer: &mut Vec<f32>) {
        let mut index: f32 = 0.0;
        let mut time_x_value = 0.0;
        let time_interval = 2.0 * f32::consts::PI / (self.samples_per_symbol - 1.0);

        while index < self.samples_per_symbol - 1.0 {
            output_buffer.push(self.execute_basis(time_x_value + phase_offset));

            index += 1.0;
            time_x_value += time_interval;
        }
    }

    fn psk_modulate(&self, input: &Vec<f32>) -> Vec<f32> {
        let mut output_buffer: Vec<f32> = Vec::with_capacity(input.len() * self.samples_per_symbol as usize);

        for input_phase in input.iter() {
            self.generate_symbol(input_phase, &mut output_buffer);
        }

        output_buffer
    }
}

impl PipelineStep<Vec<f32>, Vec<f32>> for PSKModulator {
    fn run(&mut self, input: ReceiveType<Vec<f32>>) -> Vec<f32> {
        let result = self.psk_modulate(&input);

        return result;
    }
}