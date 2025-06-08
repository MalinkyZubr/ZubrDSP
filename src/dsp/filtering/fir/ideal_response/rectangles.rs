use num::Complex;
use super::impulse_response::*;
use crate::general::validation_functions::{BoundType, is_within_bounds};


pub enum SidePassType {
    HighPass,
    LowPass
}


pub struct SidePassRectangular {
    cutoff_frequency: f32,
    pass_type: SidePassType,
}
impl SidePassRectangular {
    pub fn new(cutoff_frequency: f32, sample_rate: f32, pass_type: SidePassType) -> Self {
        assert!(cutoff_frequency >= 0.0 && cutoff_frequency <= maximum_frequency(sample_rate));
        SidePassRectangular {cutoff_frequency, pass_type}
    }
}
impl ImpulseResponse for SidePassRectangular {
    fn impulse_response_f(&self, frequency_buffer: Vec<f32>) -> Vec<Complex<f32>> {
        let mut impulse_response = vec![Complex::new(0.0, 0.0); frequency_buffer.len()];

        for (index, frequency) in frequency_buffer.iter().enumerate() {
            match self.pass_type {
                SidePassType::LowPass => {if frequency.abs() <= self.cutoff_frequency {
                    impulse_response[index] = Complex::new(1.0, 0.0);
                }}
                SidePassType::HighPass => {if frequency.abs() >= self.cutoff_frequency {
                    impulse_response[index] = Complex::new(1.0, 0.0)
                }}
            }
        }
        return impulse_response;
    }
}


pub enum SelectPassType {
    BandPass,
    BandStop
}


pub struct SelectPassRectangular {
    center_frequency: f32,
    bandwidth: f32,
    pass_type: SelectPassType
}
impl SelectPassRectangular {
    pub fn new(center_frequency: f32, bandwidth: f32, sample_rate: f32, pass_type: SelectPassType) -> Self {
        let lower_edge = center_frequency - bandwidth / 2.0;
        let upper_edge = center_frequency + bandwidth / 2.0;
        assert!(lower_edge >= 0.0 && upper_edge <= maximum_frequency(sample_rate));
        
        SelectPassRectangular {center_frequency, bandwidth, pass_type}
    }
}
impl ImpulseResponse for SelectPassRectangular {
    fn impulse_response_f(&self, frequency_buffer: Vec<f32>) -> Vec<Complex<f32>> {
        let mut impulse_response = vec![Complex::new(0.0, 0.0); frequency_buffer.len()];

        for (index, frequency) in frequency_buffer.iter().enumerate() {
            match self.pass_type {
                SelectPassType::BandPass => {
                    if is_within_bounds(
                        frequency.abs(), 
                        (
                                self.center_frequency - (self.bandwidth / 2.0),
                                self.center_frequency + (self.bandwidth / 2.0)
                        ),
                        (BoundType::Inclusive, BoundType::Inclusive)
                    ) {
                    impulse_response[index] = Complex::new(1.0, 0.0);
                }}
                SelectPassType::BandStop => {
                    if !is_within_bounds(
                        frequency.abs(),
                        (
                            self.center_frequency - (self.bandwidth / 2.0),
                            self.center_frequency + (self.bandwidth / 2.0)
                        ),
                        (BoundType::Inclusive, BoundType::Inclusive)
                    ) { impulse_response[index] = Complex::new(1.0, 0.0);
                }}
            }
        }
        return impulse_response;
    }
}