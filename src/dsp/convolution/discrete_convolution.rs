pub struct DiscreteConvolution {
    impulse_response: Vec<f32>,
    input_size: usize
}

impl DiscreteConvolution {
    pub fn new(input_size: usize) -> Self {
        DiscreteConvolution {
            impulse_response: Vec::new(),
            input_size: input_size
        }
    }
    
    fn convolve_input(&self, input: &mut Vec<f32>) {
        
    }
}