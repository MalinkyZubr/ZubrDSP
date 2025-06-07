use std::f32::consts::PI;
use super::window::*;


pub struct BartlettHann {
    coefficients: [f32; 3]
}
impl BartlettHann {
    pub fn new() -> Self {
        let coefficients: [f32; 3] = [0.62, 0.48, 0.38];
        BartlettHann {coefficients}
    }
}
impl WindowFunction for BartlettHann {
    fn window_function(&self, sample: f32, window_size: f32) -> f32 {
        return self.coefficients[0] - 
            (self.coefficients[1] * ((sample / window_size) - 0.5).abs()) - 
            (self.coefficients[2] * (2.0 * PI * sample / window_size).cos())
    }
}

