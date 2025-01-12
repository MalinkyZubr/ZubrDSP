use std::fmt;
use async_std::fs::read;
use futures::future::OrElse;
use std::collections::HashMap;
use std::thread;
use std::sync::{RwLock, Arc};

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

#[derive(Clone)]
pub struct ConvolutionalParams { // basically, more of any of these is better error correction, worse computational performance
    pub context_size: u8, // k
    pub input_bits: u8,
    pub output_polynomials: Vec<u8>, // len is num output bits. R = input / output
    pub read_mask: u8
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
                output_polynomials,
                read_mask: (2 as u8).pow(input_bits as u32) - 1
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
