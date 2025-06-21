use num::Complex;
use crate::pipeline::prototype::PipelineStep;


pub struct LinearConvolutionPadder {
    input_buffer_size: usize,
    filter_size: usize
}
impl PipelineStep<Vec<Complex<f32>>, Vec<Complex<f32>>> for LinearConvolutionPadder {
    fn run<'a>(&mut self, input: Option<Vec<Complex<f32>>>) -> Vec<Complex<f32>> {
        let mut input = input.unwrap();
        input.append(&mut vec![Complex::new(0.0, 0.0); self.filter_size - 1]);
        return input;
    }
}