use num::Complex;
use crate::dsp::fft::bit_reversal::*;
use crate::dsp::fft::fftshift::fft_shift;
use crate::dsp::filtering::fir::ideal_response::impulse_response::FIRTransferFunction;
use crate::dsp::filtering::fir::windows::window::{apply_window, WindowFunction};
use crate::pipeline::api::*;


pub struct FirFilter {
    transfer_function: Vec<Complex<f32>>
}
impl FirFilter { // the buffersize for the FIR (num coefficients) need not be equal to the input buffer, must be just less than or equal to
    pub fn new<T: WindowFunction>(fir_transfer_function: Box<dyn FIRTransferFunction>, window: T, frequency_spectrum: Vec<f32>, fft_shifted: bool, input_buffer_size: usize) -> Self {
        let mut fft_unit = FFTBitReversal::new(frequency_spectrum.len(), 1, false);
        
        let mut ideal_transfer_function = fir_transfer_function.transfer_function(frequency_spectrum);
        let ideal_impulse_response = Self::extract_impulse_response(ideal_transfer_function, fft_shifted, &mut fft_unit);
        let mut windowed_impulse_response: Vec<Complex<f32>> = apply_window(ideal_impulse_response, window)
            .iter()
            .map(|x| Complex::new(*x, 0.0))
            .collect();
        
        windowed_impulse_response.append(&mut vec![Complex::new(0.0, 0.0); input_buffer_size - 1]);
        
        let windowed_transfer_function = fft_unit.fft(windowed_impulse_response);
        
        return FirFilter {transfer_function: windowed_transfer_function}
    }
    
    fn extract_impulse_response(mut ideal_transfer_function: Vec<Complex<f32>>, fft_shifted: bool, fft_unit: &mut FFTBitReversal) -> Vec<f32> {
        if fft_shifted {
            fft_shift(&mut ideal_transfer_function);
        };

        let impulse_response = fft_unit.ifft(ideal_transfer_function);
        return impulse_response.iter().map(|x| x.re).collect(); // im components should be 0 anyway.. right? If there is a problem in future THIS IS PROBABLY WHERE IT IS.... maybe?
    }
}


impl PipelineStep<Vec<Complex<f32>>, Vec<Complex<f32>>> for FirFilter {
    fn run<'a>(&mut self, mut input: ReceiveType<Vec<Complex<f32>>>) -> Result<SendType<Vec<Complex<f32>>>, String> { // assume input is pre-padded
        match input {
            ReceiveType::Single(mut value) => {
                value.append(&mut vec![Complex::new(0.0, 0.0); self.transfer_function.len() - 1]); // need padding for linear system_response
                let mut filtered: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); value.len() + self.transfer_function.len() - 1];

                for (index, value_internal) in value.iter().enumerate() {
                    filtered[index] = value_internal * self.transfer_function[index];
                }
                
                Ok(SendType::NonInterleaved(filtered))
            }
            _ => Err(String::from("Cannot take multiple inputs"))
        }
    }
}