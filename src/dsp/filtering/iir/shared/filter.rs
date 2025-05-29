// need a way to represent characteristics of an IIR filter

use std::collections::VecDeque;


pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    BandStop
}


pub struct FilterParameters {
    pub z_domain_coefficients: Option<ZDomainCoefficients>,
    filter_type: FilterType,
    q_factor: f32,
    bandwidth: f32
}


pub struct ZDomainCoefficients {
    pub order: u8,
    pub numerator: Vec<f32>, // organized in ascending order: z^0 ... z^-n
    pub denominator: Vec<f32>
}


pub struct IIRFilterRunner {
    previous_inputs: VecDeque<f32>,
    previous_outputs: VecDeque<f32>,
    z_domain_coefficients: ZDomainCoefficients
}

impl IIRFilterRunner {
    pub fn new(coefficients: ZDomainCoefficients) -> Self {
        let mut input_deque: VecDeque<f32> = VecDeque::new();
        let mut output_deque: VecDeque<f32> = VecDeque::new();

        IIRFilterRunner::zero_convolutional_buffers(&mut input_deque, &(coefficients.numerator.len() - 1));
        IIRFilterRunner::zero_convolutional_buffers(&mut output_deque, &(coefficients.denominator.len() - 1));

        IIRFilterRunner {
            previous_inputs: input_deque,
            previous_outputs: output_deque,
            z_domain_coefficients: coefficients
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

    fn apply_iir_coefficients(&self, input_point: &f32) -> f32 {
        let mut complete_value = self.z_domain_coefficients.numerator[0] * input_point;

        for(index, nume_deno_coefficients) in 
            self.z_domain_coefficients.numerator
                .iter()
                .zip(&self.z_domain_coefficients.denominator)
                .enumerate()
                .skip(1) {
            
            let (numerator, denominator) = nume_deno_coefficients;

            complete_value += (self.previous_inputs[index - 1] * numerator) + (self.previous_outputs[index - 1] * denominator);
        }

        return complete_value;
    }

    pub fn run_iir_filter(&mut self, input: Vec<f32>) -> Vec<f32> {
        let mut filter_value;
        let mut filtered_output: Vec<f32> = Vec::with_capacity(input.len());

        for input_point in input.iter() {

            // application of coefficients and the like
            filter_value = self.apply_iir_coefficients(input_point);

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