// need a way to represent characteristics of an IIR filter

use std::collections::VecDeque;
use num::traits::Pow;


pub enum FilterType {
    LOW_PASS,
    HIGH_PASS,
    BAND_PASS,
    BAND_STOP
}


pub struct FilterParameters {
    pub z_domain_coefficients: Option<ZDomainCoefficients>,
    filter_type: FilterType,
    q_factor: f32,
    bandwidth: f32
}


pub struct ZDomainCoefficients {
    order: u8,
    numerator: Vec<f32>, // organized in ascending order: z^0 ... z^-n
    denominator: Vec<f32>
}

impl ZDomainCoefficients {
    pub fn get_numerator(&self) -> &Vec<f32> {
        return &self.numerator;
    }

    pub fn get_denominator(&self) -> &Vec<f32> {
        return &self.denominator;
    }
}


pub struct IIRFilterRunner {
    previous_inputs: VecDeque<f32>,
    previous_outputs: VecDeque<f32>
}

impl IIRFilterRunner {
    pub fn new(coefficients: &ZDomainCoefficients) -> Self {
        let mut input_deque: VecDeque<f32> = VecDeque::new();
        let mut output_deque: VecDeque<f32> = VecDeque::new();

        IIRFilterRunner::zero_convolutional_buffers(&mut input_deque, &(coefficients.numerator.len() - 1));
        IIRFilterRunner::zero_convolutional_buffers(&mut output_deque, &(coefficients.denominator.len() - 1));

        IIRFilterRunner {
            previous_inputs: input_deque,
            previous_outputs: output_deque
        }
    }

    fn zero_convolutional_buffers(deque: &mut VecDeque<f32>, size: &usize) {
        deque.clear();

        let mut index = 0;

        while index < *size {
            deque.push_back(0.0);
            index += 1;
        }
    }

    fn apply_iir_coefficients(&self, input_point: &f32, coefficients: &ZDomainCoefficients) -> f32 {
        let mut complete_value = coefficients.numerator[0] * input_point;

        for(index, nume_deno_coefficients) in 
            coefficients.numerator
                .iter()
                .zip(&coefficients.denominator)
                .enumerate()
                .skip(1) {
            
            let (numerator, denominator) = nume_deno_coefficients;

            complete_value += (self.previous_inputs[index - 1] * numerator) + (self.previous_outputs[index - 1] * denominator);
        }

        return complete_value;
    }

    pub fn run_iir_filter(&mut self, input: Vec<f32>, coefficients: &ZDomainCoefficients) -> Vec<f32> {
        let mut filter_value;
        let mut filtered_output: Vec<f32> = Vec::with_capacity(input.len());

        for input_point in input.iter() {

            // application of coefficients and the like
            filter_value = self.apply_iir_coefficients(input_point, coefficients);

            self.previous_inputs.pop_back();
            self.previous_inputs.push_front(*input_point);

            self.previous_outputs.pop_back();
            self.previous_outputs.push_front(*&filter_value);

            filtered_output.push(filter_value);
        }

        return filtered_output;
    }
}


//generalize this later
pub trait Filter {
    fn find_order(&mut self) -> f32;
    fn find_gain(&mut self) -> f32;
}

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

// pub fn filter_coefficient_corrections(numerator: &mut Vec<f32>, denominator: &mut Vec<f32>) {
//     normalize_coefficients(numerator, denominator);

//     let filter_gain = calculate_filter_gain(numerator, denominator);
//     dbg!("{}", filter_gain);

//     for index in 0..numerator.len() {
//         numerator[index] = numerator[index] * filter_gain;
//     }
// }

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