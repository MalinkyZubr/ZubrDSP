use crate::pipeline::api::*;


pub fn pointwise_arithmetic<F>(mut data: Vec<Vec<f32>>, operation: F) -> Vec<f32> 
where F: Fn(f32, f32) -> f32 {
    let mut result_vector = data.pop().unwrap();

    for index in 0..result_vector.len() {
        for data_vector in data.iter() {
            result_vector[index] = operation(data_vector[index], result_vector[index]);
        }
    }
    
    result_vector
}


pub struct PointwiseAdder {
    constant_coefficient: f32
}
impl PointwiseAdder {
    pub fn new(coefficient: f32) -> PointwiseAdder {
        PointwiseAdder { constant_coefficient: coefficient }
    }
}
impl PipelineStep<Vec<f32>, Vec<f32>> for PointwiseAdder {
    fn run_MISO(&mut self, input: Vec<Vec<f32>>) -> Result<ODFormat<Vec<f32>>, String> {
        let result_vector = pointwise_arithmetic(input, |x, y| (x + y) * self.constant_coefficient);
        Ok(ODFormat::Standard(result_vector))
    }
}


pub struct PointwiseSubtractor {}
impl PointwiseSubtractor {
    pub fn new() -> PointwiseSubtractor {
        PointwiseSubtractor {}
    }
}
impl PipelineStep<Vec<f32>, Vec<f32>> for PointwiseSubtractor {
    fn run_MISO(&mut self, input: Vec<Vec<f32>>) -> Result<ODFormat<Vec<f32>>, String> {
        let result_vector = pointwise_arithmetic(input, |x, y| x - y);

        Ok(ODFormat::Standard(result_vector))
    }
}


pub struct PointwiseMultiplier {}
impl PointwiseMultiplier {
    pub fn new() -> PointwiseMultiplier {
        PointwiseMultiplier {}
    }
}
impl PipelineStep<Vec<f32>, Vec<f32>> for PointwiseMultiplier {
    fn run_MISO(&mut self, input: Vec<Vec<f32>>) -> Result<ODFormat<Vec<f32>>, String> {
        let result_vector = pointwise_arithmetic(input, |x, y| x * y);

        Ok(ODFormat::Standard(result_vector))
    }
}


pub struct PointwiseDivider {}
impl PointwiseDivider {
    pub fn new() -> PointwiseMultiplier {
        PointwiseMultiplier {}
    }
}
impl PipelineStep<Vec<f32>, Vec<f32>> for PointwiseDivider {
    fn run_MISO(&mut self, input: Vec<Vec<f32>>) -> Result<ODFormat<Vec<f32>>, String> {
        let result_vector = pointwise_arithmetic(input, |x, y| x / y);

        Ok(ODFormat::Standard(result_vector))
    }
}