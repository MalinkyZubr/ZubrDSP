use std::f32::consts;
use num::traits::Pow;


fn find_order(min_attenuation: f32, max_attenuation: f32, stop_frequency_w: f32, pass_frequency_w: f32) -> u8 {
    let min_atten_numerator = (10.0 as f32).powf(0.1 * min_attenuation) - 1.0;
    let max_atten_numerator = (10.0 as f32).powf(0.1 * max_attenuation) - 1.0;

    let numerator = (min_atten_numerator / max_atten_numerator).log10();
    let denominator = 2.0 * (stop_frequency_w / pass_frequency_w).log10();

    return (numerator / denominator).ceil() as u8;
}

fn get_normalized_coefficients(order: u8) -> Vec<f32> {
    match order {
        1 => vec![1.0, 1.0],
        2 =>vec![1.0, consts::SQRT_2, 1.0],
        3 => vec![1.0, 2.0, 2.0, 1.0],
        _ => vec![]
    }
}

fn nonstandard_frequency_scaling(order: f32, min_attenuation: f32, max_attenuation: f32, pass_frequency_w: f32, stop_frequency_w: f32) -> f32 { // pass, stop
    let scaling_factor: f32 = 1.0;

    let lower_scaling_limit = pass_frequency_w / 
        (
            (
                (10.0 as f32).powf(0.1 * max_attenuation) - 1.0
            )
            .powf(1.0 / (2.0 * order))
        );
    
    let upper_scaling_limit: f32 = stop_frequency_w / 
        (
            (
                (10.0 as f32).powf(0.1 * max_attenuation) - 1.0
            )
            .powf(1.0 / (2.0 * order))
        );
    
    return 0.0;
}

fn low_pass_shift(pass_frequency_w: f32, order: u8) -> (Vec<f32>, Vec<f32>) { // numerator, denominator
    let mut denominator_coefficients = get_normalized_coefficients(order);
    let numerator_coefficients: Vec<f32> = vec![1.0, 0.0, 0.0];

    for (index, coefficient) in denominator_coefficients.iter_mut().enumerate() {
        *coefficient = *coefficient / (pass_frequency_w.pow(index as f32));
    };

    return (numerator_coefficients, denominator_coefficients);
}

fn high_pass_shift(pass_frequency_w: f32, order: u8) -> (Vec<f32>, Vec<f32>) {
    let (numerator, mut denominator) = low_pass_shift(pass_frequency_w, order);
    denominator.reverse();
    
    let mut numerator = vec![0.0; order as usize];
    numerator[order as usize] = 1.0;

    return (numerator, denominator);
}

// fn band_pass_shift(pass_frequency_w: f32, order: u8) -> (Vec<f32>, Vec<f32>) {

// }