use std::fmt;
use std::sync::Arc;
use async_std::fs::read;
use futures::future::OrElse;
use std::collections::HashMap;

use crate::Pipeline::node::prototype::PipelineStep;
use crate::ByteLine::codings::opts::{self, check_parity, hamming_distance};


#[derive(Clone, PartialEq, Debug)]
pub enum ConvolutionalParameterError {
    ContextSizeError(String),
    InputBitError(String),
    OutputPolynomialCountError(String),
    OutputPolynomialFmtError(String),
    OutputPolynomialCatastrophicError(String)
}

impl fmt::Display for ConvolutionalParameterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContextSizeError(e) => write!(f, "Context Size Error: {}", e),
            Self::InputBitError(e) => write!(f, "Input Bit Error: {}", e),
            Self::OutputPolynomialCountError(e) => write!(f, "Output Polynomial Count Error: {}", e),
            Self::OutputPolynomialFmtError(e) => write!(f, "Output Polynomial Fmt Error: {}", e),
            Self::OutputPolynomialCatastrophicError(e) => write!(f, "Output Polynomial Catastrophic Error: {}", e),
        }
    }
}

pub struct ConvolutionalParams { // basically, more of any of these is better error correction, worse computational performance
    pub context_size: u8, // k
    pub input_bits: u8,
    pub output_polynomials: Vec<u8>, // len is num output bits. R = input / output
}

impl ConvolutionalParams { // need to have 2 wrapper structs that implement the step trait. Most operations held in here for universality
    pub fn new(context_size: u8, input_bits: u8, output_polynomials: Vec<u8>) -> Result<ConvolutionalParams, ConvolutionalParameterError> { // remember polynomials with coefficients 0 or 1, each term representing positions in the context. Modulo 2 addition of terms. LSB represented by 1 in 1 + x + x^2 for instance
        if context_size > 8 || context_size < 2 {
            return Err(
                ConvolutionalParameterError::ContextSizeError(
                    "You must pass a context size between 8 and 2 inclusive.".to_string()
                )
            );
        }
        if input_bits > context_size - 1 || input_bits < 1 || (input_bits & (input_bits - 1) != 0) {
            return Err(
                ConvolutionalParameterError::InputBitError(
                    "You must pass an input bits size between 1 and 1 
                    less than context size, inclusive, and must be 2's power (must be 4, 2, or 1)".to_string()
                )
            );
        }
        if output_polynomials.len() > 5 || output_polynomials.len() < 2 {
            return Err(
                ConvolutionalParameterError::OutputPolynomialCountError(
                    "You must pass at least 2 and less than 5 polynomials".to_string()
                )
            );
        }
        for polynomial in output_polynomials.iter() {
            if !ConvolutionalParams::polynomial_formatting_okay(&context_size, &polynomial) {
                return Err(
                    ConvolutionalParameterError::OutputPolynomialFmtError(
                        "Polynomials cannot access nonexistant space in the context. 
                        The value of your output polynomial byte 
                        cannot exceed the aggregate of 'context-size' 
                        least significant bytes in the context".to_string()
                    )
                );
            }
        }
        if ConvolutionalParams::is_catastrophic(&output_polynomials) {
            return Err(
                ConvolutionalParameterError::OutputPolynomialCatastrophicError(
                    "Polynomials must be relatively prime. 
                    This means that the greatest common denominator 
                    of all polynomials must be equal to 1".to_string()
                )
            );
        }
        else {
            Ok(ConvolutionalParams {
                context_size,
                input_bits,
                output_polynomials
            })
        }
    }

    fn polynomial_formatting_okay(context_size: &u8, output_polynomials: &u8) -> bool {
        let two: u8 = 2;
        return output_polynomials <= &two.pow(*context_size as u32);
    }

    pub fn gcd_euclidean(values: &Vec<u8>) -> u8{
        let mut a: u8 = values[0];
        let mut b: u8 = values[1];
        
        if b > a {
            std::mem::swap(&mut a, &mut b);
        }

        while a != 0 && b != 0 {
            let remainder: u8 = a % b;
            a = b;
            b = remainder;
        }
        return a + b;
    }

    pub fn euclidean_set(input_set: &Vec<u8>) -> u8 { // repeat this 
        if input_set.len() > 1 {
            let mut output_set: Vec<u8> = Vec::new();
            let mut slice_vector: Vec<u8> = vec![0,0];
            let mut start_index: usize = 0;
            let input_set_len = input_set.len();

            while start_index < input_set_len - 1 {
                let mut relation_index = start_index + 1;
                
                while relation_index < input_set_len {
                    slice_vector[0] = input_set[start_index];
                    slice_vector[1] = input_set[relation_index];
                    let gcd = ConvolutionalParams::gcd_euclidean(&slice_vector);
                    if !output_set.contains(&gcd) {
                        if gcd != 1 {
                            dbg!("Redundancy detected, pair non minimal! {}", &slice_vector);
                        }
                        output_set.push(gcd);
                    }
                    relation_index += 1;
                }

                start_index += 1;
            }

            return ConvolutionalParams::euclidean_set(&output_set);
        }
        else {
            return input_set[0];
        }
    }

    fn is_catastrophic(output_polynomials: &Vec<u8>) -> bool { // relate every element to the other
        return ConvolutionalParams::euclidean_set(output_polynomials) != 1; // this checks if the overall set is relatively prime. What if individual pairs are still not?
    }
}

pub struct ConvolutionalOutputByteFactory {
    byte: u16,
    counter: u8,
    input_length: u8
}

impl ConvolutionalOutputByteFactory {
    pub fn new(input_length: u8) -> ConvolutionalOutputByteFactory {
        ConvolutionalOutputByteFactory {
            byte: 0,
            counter: 0,
            input_length
        }
    }

    pub fn append(&mut self, input: u8) -> Option<u8> { 
        let shifted_input: u16 = (input as u16) << self.counter;
        self.byte |= shifted_input;

        self.counter += self.input_length;

        if self.counter >= 8 {
            let return_val: Option<u8> = Some(self.byte as u8);
            self.byte >>= 8;
            self.counter = 0;
            return return_val;
        }
        else {
            return None;
        }
    }

    pub fn force_complete(&mut self) -> Option<u8> {
        if self.counter > 0 {
            let return_val: Option<u8> = Some(self.byte as u8);
            self.byte = 0;
            self.counter = 0;

            return return_val;
        }

        else {
            return None;
        }
    }
}


struct ConvolutionalLookupTable {
    pub lookup: HashMap<u8, HashMap<u8, (u8, u8)>>,
    params: ConvolutionalParams,
    context: u8
}

impl ConvolutionalLookupTable {
    pub fn new(params: ConvolutionalParams) -> ConvolutionalLookupTable {
        let mut new = ConvolutionalLookupTable {
            params,
            context: 0,
            lookup: HashMap::new()
        };

        new.generate_lookups();

        return new;
    }

    fn generate_lookups(&mut self) {
        for context in 0..((2 as u8).pow(self.params.context_size as u32)) {
            let mut subtree: HashMap<u8, (u8, u8)> = HashMap::new();
            self.generate_context_subtable(context, &mut subtree);
            self.lookup.insert(context, subtree);
        }
    }

    fn generate_context_subtable(&mut self, context_value: u8, sub_table: &mut HashMap<u8, (u8, u8)>) {
        for input in 0..((2 as u8).pow(self.params.input_bits as u32)) {
            let resultant_context = context_value | input << (self.params.context_size - self.params.input_bits);
            sub_table.insert(input, (resultant_context, self.run_polynomials(resultant_context)));
        }
    }

    fn run_polynomials(&mut self, input: u8) -> u8 {
        let mut result: u8 = 0;
        for (idx, polynomial) in self.params.output_polynomials.iter().enumerate() {
            let parity = check_parity(&(self.context & *polynomial));
            result |= parity << idx;
        }

        result
    }
}

pub struct ConvolutionalEncoder {
    params: ConvolutionalParams,
    encoding_lookup: ConvolutionalLookupTable // 0 in tuple is the resultant context, 1 is the output encoded
}

impl PipelineStep<Vec<u8>> for ConvolutionalEncoder {
    fn run(&mut self, input: Vec<u8>) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::with_capacity(input.len() * self.params.output_polynomials.len() / 8);
        let read_mask = (2 as u8).pow(self.params.input_bits as u32) - 1;
        let length = input.len();
        let mut context = 0;

        let mut output_factory = ConvolutionalOutputByteFactory::new(self.params.input_bits);

        let mut index = 0;

        while index < length { // transform all fo this into lookup table instead of live calculation
            let input_byte = input[index];
            let mut bit_count = 0;

            while bit_count < 8 { // turn this into a separate function. Lazy right now, do later
                let input_stream = (input_byte >> bit_count) & read_mask;
                let (new_context, encoded_output) = self.encoding_lookup.lookup[&context][&input_stream];
                
                match output_factory.append(
                    encoded_output
                ) {
                    Some(result) => output.push(result),
                    None => (),
                };

                context = new_context;

                bit_count += self.params.input_bits;
            }

            index += 1;
        }

        return output;
    }
}

pub struct ConvolutionalDecoderPath {
    pub path: Vec<u8>,
    pub hamming_distance: u8
}

pub struct ConvolutionalDecoder {
    params: ConvolutionalParams,
    decoding_lookup: ConvolutionalLookupTable // 0 in tuple is the resultant context, 1 is the output encoded
}

impl PipelineStep<Vec<u8>> for ConvolutionalDecoder {
    fn run(&mut self, input: Vec<u8>) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new(); // preallocate me later

        

        output
    }
}

impl ConvolutionalDecoder {
    fn single_path_match(&mut self, state: u8, input: Vec<u8>) -> ConvolutionalDecoderPath {
        let mut path = ConvolutionalDecoderPath {path: Vec::new(), hamming_distance: 0};
        let mut index = 0;
        let read_mask = (2 as u8).pow(self.params.input_bits as u32) - 1;
        let length = input.len();

        while index < length { // transform all fo this into lookup table instead of live calculation
            let input_byte = input[index];
            let mut bit_count = 0;

            while bit_count < 8 { // turn this into a separate function. Lazy right now, do later
                let output_stream = (input_byte >> bit_count) & read_mask;

                let result = self.next_state(state, output_stream);
                path.path.push(result.0);
                path.hamming_distance += result.2;

                bit_count += self.params.input_bits;
            }

            index += 1;
        }

        path
    }

    fn next_state(&self, state: u8, received_output: u8) -> (u8, u8, u8) {
        let hashmap = &self.decoding_lookup.lookup[&state];
        let mut min_distance = 10; // more than max hamming distance between 2 u8
        let mut min_output = 0;
        let mut min_input: u8 = 0;

        for (input, (new_context, output)) in hashmap.iter() {
            let distance = hamming_distance((*output, received_output));

            if distance < min_distance {
                min_distance = distance;
                min_output = *output;
                min_input = *input
            }
        }

        (min_input, min_output, min_distance)
    }
}