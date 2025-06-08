use num::Complex;
use crate::general::validation_functions::percent_error;
use rand::seq::index::sample;

pub trait ImpulseResponse {
    fn impulse_response_f(bin_index: usize) -> (f32, Complex<f32>); // Piecewise function defining behavior of the filter. What is y at this frequency x?
}

pub fn maximum_frequency(sample_rate: f32) -> f32 {
    return sample_rate / 2.0
}

pub fn get_frequency_bin_size(sample_rate: f32, buffer_size: usize) -> f32 {
    return maximum_frequency(sample_rate) / (buffer_size as f32 / 2.0);
}

pub fn get_shifted_frequency_from_index(index: usize, sample_rate: f32, buffer_size: usize) -> f32 {
    let mut frequency = index as f32 * sample_rate / buffer_size as f32;
    
    if index < buffer_size / 2 {
        frequency = 
    }
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