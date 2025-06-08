use num::Complex;
use super::impulse_response::*;


pub struct LowPassRectangular {
    cutoff_frequency: f32
}
impl LowPassRectangular {
    pub fn new(cutoff_frequency: f32) -> Self {
        LowPassRectangular {cutoff_frequency}
    }
}
impl ImpulseResponse for LowPassRectangular {
    fn impulse_response_f(bin_index: usize) -> (f32, Complex<f32>) {
        
    }
}


pub struct HighPassRectangular {
    cutoff_frequency: f32
}


pub struct BandPassRectangular {
    center_frequency: f32,
    bandwidth: f32
}


pub struct BandStopRectangular {
    center_frequency: f32,
    bandwidth: f32
}