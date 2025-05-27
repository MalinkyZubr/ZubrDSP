use num::traits::Pow;


pub fn linear_to_db(gain_value: f32) -> f32 {
    return 20.0 * gain_value.log10();
}

pub fn db_to_linear(gain_value: f32) -> f32 {
    return (10.0 as f32).powf(gain_value / 20.0);
}

pub fn analog_to_digital_frequency(selected_frequency: f32, sample_frequency: f32) -> f32 {
    let sample_period = 1.0 / sample_frequency;
    return (2.0 / sample_period) * (selected_frequency * sample_period / 2.0).atan2(1.0);
}

pub fn digital_to_analog_frequency(selected_frequency: f32, sample_frequency: f32) -> f32 { // must be called in design of the 
    let sample_period = 1.0 / sample_frequency;
    return (2.0 / sample_period) * (selected_frequency * sample_period / 2.0).tan();
}

pub fn calculate_filter_gain(numerator: &Vec<f32>, denominator: &Vec<f32>) -> f32 {
    return denominator.iter().fold(0.0, |acc, x| acc + x) / numerator.iter().fold(0.0, |acc, x| acc + x)
}

pub fn normalize_coefficients(numerator: &mut Vec<f32>, denominator: &mut Vec<f32>) {
    let normalization_factor = denominator[0];
    let mut index = 0;

    while index < numerator.len() {
        numerator[index] = numerator[index] / normalization_factor;
        denominator[index] = denominator[index] / normalization_factor;

        index += 1;
    }
}

pub fn compute_z_coefficients_o1(numerator: [f32; 2], denominator: [f32; 2], sample_frequency_hz: f32) -> (Vec<f32>, Vec<f32>) {
    let constant_w = 2.0 * sample_frequency_hz;
    let mut out_numerator: Vec<f32> = Vec::with_capacity(2);
    let mut out_denominator: Vec<f32> = Vec::with_capacity(2);

    out_numerator.push((numerator[1] * constant_w) + numerator[0]);
    out_numerator.push((-numerator[1] * constant_w) + numerator[0]);

    out_denominator.push((denominator[1] * constant_w) + denominator[0]);
    out_denominator.push((-denominator[1] * constant_w) + denominator[0]);

    normalize_coefficients(&mut out_numerator, &mut out_denominator);

    return (out_numerator, out_denominator);
}

pub fn compute_z_coefficients_o2(numerator: [f32; 3], denominator: [f32; 3], sample_frequency_hz: f32) -> (Vec<f32>, Vec<f32>) {
    let constant_w = 2.0 * sample_frequency_hz;
    let constant_w_pow2 = constant_w.pow(2);
    let mut out_numerator: Vec<f32> = Vec::with_capacity(3);
    let mut out_denominator: Vec<f32> = Vec::with_capacity(3);

    out_numerator.push((numerator[2] * constant_w_pow2) + (numerator[1] * constant_w) + numerator[0]);
    out_numerator.push((-2.0 * numerator[2] * constant_w_pow2) + (2.0 * numerator[0]));
    out_numerator.push((numerator[2] * constant_w_pow2) - (numerator[1] * constant_w) + numerator[0]);

    out_denominator.push((denominator[2] * constant_w_pow2) + (denominator[1] * constant_w) + denominator[0]);
    out_denominator.push((-2.0 * denominator[2] * constant_w_pow2) + (2.0 * denominator[0]));
    out_denominator.push((denominator[2] * constant_w_pow2) - (denominator[1] * constant_w) + denominator[0]);

        dbg!("{}", &out_denominator);

    normalize_coefficients(&mut out_numerator, &mut out_denominator);

    return (out_numerator, out_denominator);
}

pub fn compute_z_coefficients_o3(numerator: [f32; 4], denominator: [f32; 4], sample_frequency_hz: f32) -> (Vec<f32>, Vec<f32>) {
    let constant_w = 2.0 * sample_frequency_hz;
    let constant_w_pow2 = constant_w.pow(2);
    let mut out_numerator: Vec<f32> = Vec::with_capacity(4);
    let mut out_denominator: Vec<f32> = Vec::with_capacity(4);

    out_numerator.push((numerator[3] * constant_w.pow(3)) + (numerator[2] * constant_w_pow2) + (numerator[1] * constant_w) + numerator[0]);
    out_numerator.push((-3.0 * numerator[3] * constant_w.pow(3)) + (-1.0 * numerator[2] * constant_w_pow2) + (numerator[1] * constant_w) + (3.0 * numerator[0]));
    out_numerator.push((3.0 * numerator[3] * constant_w.pow(3)) + (-1.0 * numerator[2] * constant_w_pow2) + (-1.0 * numerator[1] * constant_w) + (3.0 * numerator[0]));
    out_numerator.push((-1.0 * numerator[3] * constant_w.pow(3)) + (numerator[2] * constant_w_pow2) + (-1.0 * numerator[1] * constant_w) + (numerator[0]));

    out_denominator.push((denominator[3] * constant_w.pow(3)) + (denominator[2] * constant_w_pow2) + (denominator[1] * constant_w) + denominator[0]);
    out_denominator.push((-3.0 * denominator[3] * constant_w.pow(3)) + (-1.0 * denominator[2] * constant_w_pow2) + (denominator[1] * constant_w) + (3.0 * denominator[0]));
    out_denominator.push((3.0 * denominator[3] * constant_w.pow(3)) + (-1.0 * denominator[2] * constant_w_pow2) + (-1.0 * denominator[1] * constant_w) + (3.0 * denominator[0]));
    out_denominator.push((-1.0 * denominator[3] * constant_w.pow(3)) + (denominator[2] * constant_w_pow2) + (-1.0 * denominator[1] * constant_w) + (denominator[0]));

    normalize_coefficients(&mut out_numerator, &mut out_denominator);

    return (out_numerator, out_denominator);
}