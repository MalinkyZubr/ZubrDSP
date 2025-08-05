use crate::pipeline::api::*;
use std::f32::consts::PI;


pub struct VCO {
    previous_phase_sum: f32,
}
impl VCO {
    pub fn process_input_vector(&mut self, input_vector: &mut Vec<f32>) {
        input_vector[0] = self.previous_phase_sum + input_vector[0];
        let mut sum_value = input_vector[0];
        
        for input_value in input_vector.iter_mut() {
            sum_value = (sum_value + *input_value) % (2.0 * PI);
            *input_value = sum_value;
        }
        
        self.previous_phase_sum = sum_value;
    }
}

impl PipelineStep<Vec<f32>, Vec<f32>> for VCO {
    fn run_SISO(&mut self, mut input: Vec<f32>) -> Result<ODFormat<Vec<f32>>, String> {
        self.process_input_vector(&mut input);
        Ok(ODFormat::Standard(input))
    }
}