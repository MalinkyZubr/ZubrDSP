use std::collections::HashMap;
use super::params::ConvolutionalParams;
use crate::ByteLine::codings::opts::{check_parity};

pub struct ConvolutionalLookupTable {
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