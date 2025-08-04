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


pub struct PointwiseAdder {}
impl PointwiseAdder {
    pub fn new() -> PointwiseAdder {
        PointwiseAdder {}
    }
}
impl PipelineStep<Vec<f32>, Vec<f32>> for PointwiseAdder {
    fn run(&mut self, input: ReceiveType<Vec<f32>>) -> Result<SendType<Vec<f32>>, String> {
        match input {
            ReceiveType::Multi(data) => {
                let result_vector = pointwise_arithmetic(data, |x, y| x + y);
                
                Ok(SendType::NonInterleaved(result_vector))
            },
            _ => Err("Expecting multiple vector inputs fopr pointwise adder".to_string()),
        }
    }
}


pub struct PointwiseSubtractor {}
impl PointwiseSubtractor {
    pub fn new() -> PointwiseSubtractor {
        PointwiseSubtractor {}
    }
}
impl PipelineStep<Vec<f32>, Vec<f32>> for PointwiseSubtractor {
    fn run(&mut self, input: ReceiveType<Vec<f32>>) -> Result<SendType<Vec<f32>>, String> {
        match input {
            ReceiveType::Multi(mut data) => {
                let result_vector = pointwise_arithmetic(data, |x, y| x - y);

                Ok(SendType::NonInterleaved(result_vector))
            },
            _ => Err("Expecting multiple vector inputs fopr pointwise adder".to_string()),
        }
    }
}


pub struct PointwiseMultiplier {}
impl PointwiseMultiplier {
    pub fn new() -> PointwiseMultiplier {
        PointwiseMultiplier {}
    }
}
impl PipelineStep<Vec<f32>, Vec<f32>> for PointwiseMultiplier {
    fn run(&mut self, input: ReceiveType<Vec<f32>>) -> Result<SendType<Vec<f32>>, String> {
        match input {
            ReceiveType::Multi(mut data) => {
                let result_vector = pointwise_arithmetic(data, |x, y| x * y);

                Ok(SendType::NonInterleaved(result_vector))
            },
            _ => Err("Expecting multiple vector inputs fopr pointwise adder".to_string()),
        }
    }
}


pub struct PointwiseDivider {}
impl PointwiseDivider {
    pub fn new() -> PointwiseMultiplier {
        PointwiseMultiplier {}
    }
}
impl PipelineStep<Vec<f32>, Vec<f32>> for PointwiseDivider {
    fn run(&mut self, input: ReceiveType<Vec<f32>>) -> Result<SendType<Vec<f32>>, String> {
        match input {
            ReceiveType::Multi(mut data) => {
                let result_vector = pointwise_arithmetic(data, |x, y| x / y);

                Ok(SendType::NonInterleaved(result_vector))
            },
            _ => Err("Expecting multiple vector inputs fopr pointwise adder".to_string()),
        }
    }
}