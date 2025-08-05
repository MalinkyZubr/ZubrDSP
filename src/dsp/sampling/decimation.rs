use crate::pipeline::api::*;

pub struct Decimator {
    sampling_period: u32 // every nth sample is kept
}
impl Decimator {
    pub fn new(sampling_period: u32) -> Decimator {
        Decimator { sampling_period }
    }
    pub fn decimate(&self, samples: &Vec<f32>) -> Vec<f32> {
        let mut output_vector = Vec::with_capacity(samples.len() / self.sampling_period as usize);
        
        for value in samples.iter().step_by(self.sampling_period as usize) {
            output_vector.push(*value);
        }
        
        output_vector
    }
}
impl PipelineStep<Vec<f32>, Vec<f32>> for Decimator {
    fn run_SISO(&mut self, input: Vec<f32>) -> Result<ODFormat<Vec<f32>>, String> { 
        Ok(ODFormat::Standard(input))
    }
}