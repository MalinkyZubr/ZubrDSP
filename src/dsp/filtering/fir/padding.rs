use num::Complex;
use crate::pipeline::api::*;


pub struct LinearConvolutionPadder {
    input_buffer_size: usize,
    filter_size: usize
}
impl PipelineStep<Vec<Complex<f32>>, Vec<Complex<f32>>> for LinearConvolutionPadder {
    fn run<'a>(&mut self, mut input: ReceiveType<Vec<Complex<f32>>>) -> Result<SendType<Vec<Complex<f32>>>, String> {
        match input {
            ReceiveType::Single(mut value) => {
                value.append(&mut vec![Complex::new(0.0, 0.0); self.filter_size - 1]);
                Ok(SendType::NonInterleaved(value))
            },
            _ => Err(String::from("Cannot take multiple inputs"))
        }
    }
}