use crate::pipeline::api::*;
use crate::pipeline::api::ReceiveType::Single;
use crate::pipeline::api::SendType::NonInterleaved;

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
    fn run(&mut self, input: ReceiveType<Vec<f32>>) -> Result<SendType<Vec<f32>>, String> {
        match input {
            Single(value) => Ok(SendType::NonInterleaved(self.insert_0_samples(&value))),
            _ => Err("Upsampler doesn't support multi input.".to_string())
        }
    }
}