use num::Complex;
use crate::pipeline::node::prototype::PipelineStep;


pub struct LinearConvolutionPadder {
    input_buffer_size: usize,
    filter_size: usize
}
impl PipelineStep<Vec<Complex<f32>>, Vec<Complex<f32>>> for LinearConvolutionPadder {
    fn run<'a>(&mut self, mut input: Vec<Complex<f32>>) -> Vec<Complex<f32>> {
        input.append(&mut vec![Complex::new(0.0, 0.0); self.filter_size - 1]);
        return input;
    }
}