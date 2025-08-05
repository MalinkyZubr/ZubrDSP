use crate::pipeline::api::*;
use crate::pipeline::api::ReceiveType::Single;
use crate::pipeline::api::*;

pub struct Upsampler {
    upsample_factor: usize // there will be n * upsample_factor samples after. For every sample add upsample_factor - 1 samples
}
impl Upsampler { // note, I only insert the 0 samples. Still need a low pass filter
    pub fn new(upsample_factor: usize) -> Upsampler {
        Upsampler {
            upsample_factor
        }
    }
    
    pub fn insert_0_samples(&self, samples: &Vec<f32>) -> Vec<f32> {
        let mut output = Vec::with_capacity(samples.len() * self.upsample_factor);
        
        for sample in samples.iter() {
            output.push(*sample);
            output.extend(vec!(0.0; self.upsample_factor - 1));
        }
        
        output
    }
}
impl PipelineStep<Vec<f32>, Vec<f32>> for Upsampler {
    fn run_SISO(&mut self, input: Vec<f32>) -> Result<ODFormat<Vec<f32>>, String> {
        Ok(ODFormat::Standard(self.insert_0_samples(&input)))
    }
}