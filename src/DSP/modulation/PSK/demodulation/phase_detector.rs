use std::f32;


pub fn generate_phase_diff(input_signal: &Vec<f32>, reference: &Vec<f32>) -> Vec<f32> { // must low pass filter the result of this
    let phase_difference_vector = input_signal
        .iter()
        .zip(reference.iter())
        .map(|(&x, &y)| x * y)
        .collect::<Vec<f32>>();

    phase_difference_vector
}

pub fn vectorized_inverse_cosine(input_signal: &Vec<f32>) -> Vec<f32> {
    let inverse_cosine_vector = input_signal
        .iter()
        .map(|&x| f32::acos(x))
        .collect::<Vec<f32>>();

    inverse_cosine_vector
}

