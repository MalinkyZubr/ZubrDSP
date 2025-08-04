use num::Complex;
use std::f32::consts::PI;
use crate::general::validation_functions::percent_error;
use rand::seq::index::sample;
use crate::dsp::system_response::system_functions::TransferFunction;

pub trait FIRFilter {
    fn transfer_function(&self, frequency_buffer: Vec<f32>) -> TransferFunction; // Piecewise function defining behavior of the filter. What is y at this frequency x?
}

pub fn generate_frequency_buffer(buffer_size: usize, sample_frequency: f32) -> Vec<f32> {
    let mut pos: Vec<i64> = (0..(buffer_size as i64 / 2) + 1).collect();
    let neg: Vec<i64> = (-(buffer_size as i64 / 2) - 1..-0).collect();

    pos.extend_from_slice(neg.as_slice());

    let acc = pos.iter().map(|n| *n as f32 * sample_frequency / buffer_size as f32).collect();

    acc
}

pub fn maximum_frequency(sample_rate: f32) -> f32 {
    return sample_rate / 2.0
}

pub fn get_frequency_bin_size(sample_rate: f32, buffer_size: usize) -> f32 {
    return maximum_frequency(sample_rate) / (buffer_size as f32 / 2.0);
}

pub fn get_frequency_from_index(index: usize, sample_rate: f32, buffer_size: usize) -> f32 {
    let mut frequency = index as f32 * sample_rate / buffer_size as f32;
    return frequency;
}

pub fn get_best_shifted_bin_index(desired_frequency: f32, sample_rate: f32, buffer_size: usize) -> usize {
    assert!(desired_frequency <= maximum_frequency(sample_rate));
    //let bin_size = get_frequency_bin_size(sample_rate, buffer_size);
    
    let shifted_index_raw = desired_frequency * buffer_size as f32 / sample_rate;
    let shifted_index: usize = (shifted_index_raw.ceil()) as usize;
    
    let approximate_frequency = shifted_index as f32 * sample_rate / buffer_size as f32;
    if percent_error(approximate_frequency, desired_frequency) > 4.0 {
        dbg!(
            "Warning: desired frequency {} deviates from assigned {} by {} %%. Consider increasing FFT resolution!",
            &desired_frequency,
            &approximate_frequency,
            &buffer_size
        );
    }
    
    return (buffer_size / 2) + shifted_index;
}

