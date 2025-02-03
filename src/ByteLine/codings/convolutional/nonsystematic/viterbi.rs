use std::collections::HashMap;

use crate::ByteLine::codings::opts::hamming_distance;
use crate::Pipeline::node::prototype::PipelineStep;
use super::params::ConvolutionalParams;
use super::trellis::{ConvolutionalDecoderLookup, TrellisStateChangeDecode};
use super::encoder_io::{ConvolutionalInputProcessor, ConvolutionalOutputByteFactory, ConvolutionalInputConsumer};


struct ViterbiProcessor {}
impl ConvolutionalInputProcessor for ViterbiProcessor {
    fn process(&mut self, stream: u8) -> Option<u8> { // why not autogenerate the extracted symbol sequence and then send that to each path analyzer? redundant in current state
        return Some(stream);
    }
}

#[derive(Clone)]
struct TrellisRoute {
    pub route: Vec<TrellisStateChangeDecode>,
    hamming_distance: u8
}

struct ViterbiTransfer {
    future_time: HashMap<u8, Option<TrellisRoute>>,
    decoder_lookup: ConvolutionalDecoderLookup,
}

impl ViterbiTransfer {
    pub fn flush_future_state_tree(&mut self) {
        for key in self.future_time.keys() {
            self.future_time[key] = None;
        }
    }

    fn identify_best_source(&mut self, input: &u8, future_state: &u8, current_time: HashMap<u8, Option<TrellisRoute>>) -> TrellisRoute { // doing it this way to minimize memory reallocations due to clones
        let mut best_source: u8 = 0;
        let mut best_distance: u8 = 254;
        for (current_state, transition) in self.decoder_lookup.state_transition(*future_state).iter() {
            match current_time[current_state] {
                Some(route) => {
                    let distance = hamming_distance((*input, transition.output)) + route.hamming_distance;

                    if distance < best_distance {
                        best_source = *current_state;
                        best_distance = distance;
                    }
                }
                None => (),
            };
        }

        let mut best_route = current_time[&best_source].clone().unwrap();
        best_route.hamming_distance = best_distance;

        return best_route;
    }

    pub fn update_routes(&mut self, input: &u8, current_time: HashMap<u8, Option<TrellisRoute>>) -> HashMap<u8, Option<TrellisRoute>> {
        for future_state in self.future_time.keys() {
            self.future_time[future_state] = Some(self.identify_best_source(input, future_state, current_time));
        }

        let new_current_time = self.future_time;
        self.flush_future_state_tree();

        return new_current_time;
    }
}

pub struct ConvolutionalDecoder {
    output_factory: ConvolutionalOutputByteFactory,
    input_consumer: ConvolutionalInputConsumer,
    params: ConvolutionalParams,
    viterbi_transfer: ViterbiTransfer
}

impl ConvolutionalDecoder {
    pub fn new(params: ConvolutionalParams, num_threads: usize, decoding_lookup: ConvolutionalDecoderLookup) -> ConvolutionalDecoder { // make it so the lookup is generated in here with references and all, etc
        let state_tree: HashMap<u8, Option<TrellisRoute>> = decoding_lookup.generate_state_vec()
            .into_iter()
            .map(|key| (key, None))
            .collect();
        
        ConvolutionalDecoder {
            params: params.clone(),
            output_factory: ConvolutionalOutputByteFactory::new(params.input_bits),
            input_consumer: ConvolutionalInputConsumer::new(Box::new(ViterbiProcessor{}), params.clone()), // processor is an asset of the consumer. Should ahve to go through consumer to access processor. So why does this struct maintain ownership? fix
            viterbi_transfer: ViterbiTransfer {future_time: state_tree, decoder_lookup: decoding_lookup}
            //parallelized_decoders: Vec::with_capacity(num_threads),
        }
    }

    fn trellis_route_to_stream(&mut self, input: &TrellisRoute) -> Vec<u8> {
        let mut re_assembled_input: Vec<u8> = Vec::with_capacity(self.params.input_bits as usize * input.route.len() / 8);

        for state in input.route.iter() {
            let byte = self.output_factory.append(state.input);

            match byte {
                Some(new_byte) => re_assembled_input.push(new_byte),
                None => continue,
            };
        };

        match self.output_factory.force_complete() {
            Some(final_byte) => re_assembled_input.push(final_byte),
            None => (),
        };

        return re_assembled_input;
    }

    fn get_best_route(&self, final_states: HashMap<u8, Option<TrellisRoute>>) -> &TrellisRoute {
        let mut best_distance = 254;
        let mut best_route: &TrellisRoute;

        for route in final_states.values() {
            match route {
                Some(valid_route) => {
                    if valid_route.hamming_distance < best_distance {
                        best_distance = valid_route.hamming_distance;
                        best_route = &valid_route;
                    }
                }
                None => (),
            };
        };

        return &best_route;
    }

    fn viterbi(&mut self, input: Vec<u8>) -> Vec<u8> {
        let mut new_states: HashMap<u8, Option<TrellisRoute>> = HashMap::new();
        for output_bitstream in input.iter() {
            new_states = self.viterbi_transfer.update_routes(output_bitstream, new_states);
        };

        let final_route: &TrellisRoute = self.get_best_route(new_states);
        return self.trellis_route_to_stream(final_route);
    }
}

impl PipelineStep<Vec<u8>> for ConvolutionalDecoder {
    fn run(&mut self, input: Vec<u8>) -> Vec<u8> {
        let separated = self.input_consumer.consume(&input);
        self.viterbi(separated)
    }
}

