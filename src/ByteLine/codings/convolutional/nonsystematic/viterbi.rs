use crate::ByteLine::codings::opts::hamming_distance;
use crate::Pipeline::node::prototype::PipelineStep;
use super::params::ConvolutionalParams;
use super::trellis::ConvolutionalLookupTable;
use super::encoder_io::{ConvolutionalInputProcessor, ConvolutionalOutputByteFactory, ConvolutionalInputConsumer};


pub struct ViterbiProcessor {
    hamming_distance: u8,
    params: ConvolutionalParams,// 0 in tuple is the resultant context, 1 is the output encoded
    //parallelized_decoders: Vec<thread::JoinHandle<()>> // implement this paralellization later. Lets make sure this works at all first!
    context: (u8, u8, u8),
    decoding_lookup: ConvolutionalLookupTable,
}

impl ViterbiProcessor {
    pub fn new(params: ConvolutionalParams, decoding_lookup: ConvolutionalLookupTable) -> Self {
        ViterbiProcessor {
            hamming_distance: 0,
            params,
            decoding_lookup,
            context: (0, 0, 0),
        }
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

impl ConvolutionalInputProcessor for ViterbiProcessor {
    fn process(&mut self, stream: u8) -> Option<u8> { // why not autogenerate the extracted symbol sequence and then send that to each path analyzer? redundant in current state
        let result = self.next_state(self.context.0, stream);
        self.context = result;

        self.hamming_distance += result.2;

        return Some(result.0);
    }
}

pub struct ConvolutionalDecoder {
    params: ConvolutionalParams,// 0 in tuple is the resultant context, 1 is the output encoded
    //parallelized_decoders: Vec<thread::JoinHandle<()>> // implement this paralellization later. Lets make sure this works at all first!
    output_factory: ConvolutionalOutputByteFactory,
    input_consumer: ConvolutionalInputConsumer
}

impl ConvolutionalDecoder {
    pub fn new(params: ConvolutionalParams, num_threads: usize, decoding_lookup: ConvolutionalLookupTable) -> ConvolutionalDecoder { // make it so the lookup is generated in here with references and all, etc
        ConvolutionalDecoder {
            output_factory: ConvolutionalOutputByteFactory::new(params.input_bits),
            params,
            input_consumer: ConvolutionalInputConsumer::new(Box::new(ViterbiProcessor::new(params.clone(), )))
            //parallelized_decoders: Vec::with_capacity(num_threads),
        }
    }
}

impl PipelineStep<Vec<u8>> for ConvolutionalDecoder {
    fn run(&mut self, input: Vec<u8>) -> Vec<u8> {
        let mut possible_outputs: Vec<ConvolutionalDecoderPath> = Vec::with_capacity(self.decoding_lookup.lookup.len());
        for state in self.decoding_lookup.lookup.into_keys().iter() {

        }
    }
}