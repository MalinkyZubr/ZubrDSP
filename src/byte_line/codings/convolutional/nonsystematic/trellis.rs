use std::collections::HashMap;

use super::{params::ConvolutionalParams};
use crate::byte_line::codings::opts::{check_parity};


pub type TrellisState = u8;
pub type TrellisInput = u8;
pub type TrellisOutput = u8;


#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TrellisStateChangeEncode {
    pub new_state: u8,
    pub output: u8
}

pub struct ConvolutionalEncoderLookup {
    pub encoding_lookup: HashMap<TrellisState, HashMap<TrellisInput, TrellisStateChangeEncode>>,
    pub output_size: u8
}

impl ConvolutionalEncoderLookup {
    pub fn state_transition(&self, current_state: u8, input: u8) -> TrellisStateChangeEncode {
        self.encoding_lookup[&current_state][&input]
    }

    pub fn to_transition_matrix(&self) -> Vec<Vec<u8>> {
        let num_states: u8 = self.encoding_lookup.len() as u8;
        let mut transition_matrix: Vec<Vec<u8>> = vec![vec![0; num_states as usize]; num_states as usize];

        for state in 0..num_states {
            let probability: u8 = 1; // modification of the traditional hideen markov model
            // in the context of error correction, the state transition probability is irrelevant. For arbitrary input all transitions are equally probable

            for transition in &self.encoding_lookup[&state] {
                transition_matrix[state as usize][transition.1.new_state as usize] = probability;
            }
        };

        return transition_matrix;
    }

    pub fn to_emission_matrix(&self) -> Vec<Vec<u16>> {
        let num_states: u8 = self.encoding_lookup.len() as u8;
        let mut emission_matrix: Vec<Vec<u16>> = vec![vec![256; (2 as u64).pow(self.output_size as u32) as usize]; num_states as usize];

        for state in 0..num_states {
            for transition in &self.encoding_lookup[&state] {
                emission_matrix[state as usize][transition.1.new_state as usize] = transition.1.output as u16;
            }
        };

        return emission_matrix;
    }
}

pub struct ConvolutionalLookupGenerator {
    params: ConvolutionalParams,
}

impl ConvolutionalLookupGenerator {
    pub fn generate_encoding_lookup(params: &ConvolutionalParams) -> ConvolutionalEncoderLookup {
        let mut lookup_table: HashMap<TrellisState, HashMap<TrellisInput, TrellisStateChangeEncode>> = HashMap::new();

        for state in 0..(params.max_state_mask + 1) {
            let state_changes = Self::generate_state_changes_encode(state, params);
            lookup_table.insert(state, state_changes);
        };

        ConvolutionalEncoderLookup {encoding_lookup: lookup_table, output_size: params.output_polynomials.len() as u8}
    }

    fn generate_state_changes_encode(state: u8, params: &ConvolutionalParams) -> HashMap<TrellisInput, TrellisStateChangeEncode> {
        let mut state_changes: HashMap<TrellisInput, TrellisStateChangeEncode> = HashMap::new();
        let offset_state = (state << params.input_bits) & params.max_state_mask;

        for input in 0..(params.read_mask + 1) { // do not exclude the max input
            let new_state = offset_state | input;
            state_changes.insert(input, TrellisStateChangeEncode {new_state, output: Self::run_polynomials(new_state, params)});
        };

        state_changes
    }

    fn run_polynomials(new_state: u8, params: &ConvolutionalParams) -> u8 {
        let mut result: u8 = 0;
        for (idx, polynomial) in params.output_polynomials.iter().enumerate() {
            let parity = check_parity(&(new_state & *polynomial));
            result |= parity << idx; // getting the sum of polynomial results with coefficients is equal to parity of the statement
        }

        result
    }
}