use std::collections::HashMap;
use async_std::task::current;

use super::{encoder, params::ConvolutionalParams};
use crate::ByteLine::codings::opts::{check_parity};


pub type TrellisState = u8;
pub type TrellisInput = u8;
pub type TrellisOutput = u8;


#[derive(Clone, Copy)]
pub struct TrellisStateChangeEncode {
    pub new_state: u8,
    pub output: u8
}

#[derive(Clone, Copy)]
pub struct TrellisStateChangeDecode {
    pub output: u8,
    pub input: u8,
}

pub struct ConvolutionalEncoderLookup {
    encoding_lookup: HashMap<TrellisState, HashMap<TrellisInput, TrellisStateChangeEncode>>,
}

impl ConvolutionalEncoderLookup {
    pub fn state_transition(&self, current_state: u8, input: u8) -> TrellisStateChangeEncode {
        self.encoding_lookup[&current_state][&input]
    }
}

pub struct ConvolutionalDecoderLookup {
    decoding_lookup: HashMap<TrellisState, HashMap<TrellisOutput, TrellisStateChangeDecode>>,
}

impl ConvolutionalDecoderLookup {
    pub fn state_transition(&self, current_state: u8) -> HashMap<TrellisState, TrellisStateChangeDecode> {
        self.decoding_lookup[&current_state].clone()
    }

    pub fn generate_state_vec(&self) -> Vec<TrellisState> {
        self.decoding_lookup.keys().cloned().collect()
    }
}

pub struct ConvolutionalLookupGenerator {
    params: ConvolutionalParams,
}

impl ConvolutionalLookupGenerator {
    pub fn generate_encoding_lookup(params: &ConvolutionalParams) -> ConvolutionalEncoderLookup {
        let mut lookup_table: HashMap<TrellisState, HashMap<TrellisInput, TrellisStateChangeEncode>> = HashMap::new();

        for state in 0..((2 as u8).pow(params.context_size as u32)) {
            let state_changes = Self::generate_state_changes_encode(state, params);
            lookup_table.insert(state, state_changes);
        };

        ConvolutionalEncoderLookup {encoding_lookup: lookup_table}
    }

    fn generate_state_changes_encode(state: u8, params: &ConvolutionalParams) -> HashMap<TrellisInput, TrellisStateChangeEncode> {
        let mut state_changes: HashMap<TrellisInput, TrellisStateChangeEncode> = HashMap::new();
        for input in 0..((2 as u8).pow(params.input_bits as u32)) {
            let new_state = state | input << (params.context_size - params.input_bits);
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

    fn generate_state_changes_decode(state: u8, params: &ConvolutionalParams) -> HashMap<TrellisState, TrellisStateChangeDecode> {
        let mut state_changes: HashMap<TrellisState, TrellisStateChangeDecode> = HashMap::new();
        for input in 0..((2 as u8).pow(params.input_bits as u32)) {
            let old_state = (state << (params.context_size - params.input_bits)) | input;
            state_changes.insert(input, TrellisStateChangeDecode {input, output: Self::run_polynomials(state, params)});
        };

        state_changes
    }

    pub fn generate_decoding_lookup(params: &ConvolutionalParams) -> ConvolutionalDecoderLookup {
        let mut lookup_table: HashMap<TrellisState, HashMap<TrellisState, TrellisStateChangeDecode>> = HashMap::new();

        for state in 0..((2 as u8).pow(params.context_size as u32)) {
            let state_changes: HashMap<TrellisState, TrellisStateChangeDecode> = Self::generate_state_changes_decode(state, params);
            lookup_table.insert(state, state_changes);
        };

        ConvolutionalDecoderLookup {decoding_lookup: lookup_table}
    }
}