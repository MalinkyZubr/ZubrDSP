use num::Complex;
use super::fir_utils::*;
use crate::dsp::system_response::system_functions::TransferFunction;
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
impl FIRFilter for SidePassRectangular {
    fn transfer_function(&self, frequency_buffer: Vec<f32>) -> TransferFunction {
        let mut transfer_window = vec![Complex::new(0.0, 0.0); frequency_buffer.len()];

        for (index, frequency) in frequency_buffer.iter().enumerate() {
            match self.pass_type {
                SidePassType::LowPass => {if frequency.abs() <= self.cutoff_frequency {
                    transfer_window[index] = Complex::new(1.0, 0.0);
                }}
                SidePassType::HighPass => {if frequency.abs() >= self.cutoff_frequency {
                    transfer_window[index] = Complex::new(1.0, 0.0)
                }}
            }
        }
        TransferFunction::new_configured(transfer_window)
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
impl FIRFilter for SelectPassRectangular {
    fn transfer_function(&self, frequency_buffer: Vec<f32>) -> TransferFunction {
        let mut transfer_window = vec![Complex::new(0.0, 0.0); frequency_buffer.len()];

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
                    transfer_window[index] = Complex::new(1.0, 0.0);
                }}
                SelectPassType::BandStop => {
                    if !is_within_bounds(
                        frequency.abs(),
                        (
                            self.center_frequency - (self.bandwidth / 2.0),
                            self.center_frequency + (self.bandwidth / 2.0)
                        ),
                        (BoundType::Inclusive, BoundType::Inclusive)
                    ) { transfer_window[index] = Complex::new(1.0, 0.0);
                }}
            }
        }
        TransferFunction::new_configured(transfer_window)
    }
}