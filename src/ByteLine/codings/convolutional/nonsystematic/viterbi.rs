use crate::ByteLine::codings::opts::hamming_distance;
use crate::Pipeline::node::prototype::PipelineStep;
use super::params::ConvolutionalParams;
use super::trellis::ConvolutionalLookupTable;
use super::encoder_io::{ConvolutionalInputProcessor, ConvolutionalOutputByteFactory, ConvolutionalInputConsumer};


struct ViterbiProcessor {
    context: u8,
    output_factory: ConvolutionalOutputByteFactory,
    decoding_lookup: ConvolutionalLookupTable,
}

impl ConvolutionalInputProcessor<u8> for ViterbiProcessor {
    fn process(&mut self, stream: u8) -> Option<u8> {
        
    }
}

impl ViterbiProcessor {
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


pub struct ConvolutionalDecoderPath {
    pub path: Vec<u8>,
    pub hamming_distance: u8,
}

pub struct ConvolutionalDecoder {
    params: ConvolutionalParams,// 0 in tuple is the resultant context, 1 is the output encoded
    //parallelized_decoders: Vec<thread::JoinHandle<()>> // implement this paralellization later. Lets make sure this works at all first!
}


impl PipelineStep<Vec<u8>> for ConvolutionalDecoder {
    fn run(&mut self, input: Vec<u8>) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new(); // preallocate me later

        

        output
    }
}

impl ConvolutionalDecoder {
    pub fn new(params: ConvolutionalParams, num_threads: usize, decoder_lookup: ConvolutionalLookupTable) -> ConvolutionalDecoder { // make it so the lookup is generated in here with references and all, etc
        ConvolutionalDecoder {
            params: params,
            decoding_lookup: decoder_lookup,
            //parallelized_decoders: Vec::with_capacity(num_threads),
        }
    }

    fn single_path_match(&mut self, state: u8, input: Vec<u8>) -> ConvolutionalDecoderPath {
        let mut path = ConvolutionalDecoderPath {path: Vec::new(), hamming_distance: 0};
        let mut index = 0;
        let length = input.len();

        while index < length { // transform all fo this into lookup table instead of live calculation
            let input_byte = input[index];
            let mut bit_count = 0;

            while bit_count < 8 { // turn this into a separate function. Lazy right now, do later
                let output_stream = (input_byte >> bit_count) & self.params.read_mask;

                let result = self.next_state(state, output_stream);
                path.path.push(result.0);
                path.hamming_distance += result.2;

                bit_count += self.params.input_bits;
            }

            index += 1;
        }

        path
    }
}