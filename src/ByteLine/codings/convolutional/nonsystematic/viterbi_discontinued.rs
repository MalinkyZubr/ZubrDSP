use std::collections::HashMap;

use async_std::task::current;

use crate::ByteLine::codings::opts::hamming_distance;
use crate::Pipeline::node::prototype::PipelineStep;
use super::encoder;
use super::params::ConvolutionalParams;
use super::trellis::{ConvolutionalDecoderLookup, ConvolutionalLookupGenerator, TrellisStateChangeDecode};
use super::encoder_io::{ConvolutionalInputProcessor, ConvolutionalOutputByteFactory, ConvolutionalInputConsumer};


struct ViterbiProcessor {}
impl ConvolutionalInputProcessor for ViterbiProcessor {
    fn process(&mut self, stream: u8) -> Option<u8> { // why not autogenerate the extracted symbol sequence and then send that to each path analyzer? redundant in current state
        return Some(stream);
    }
}

#[derive(Clone, Debug)]
struct TrellisRoute {
    pub route: Vec<TrellisStateChangeDecode>,
    hamming_distance: u8
}

struct ViterbiTransfer {
    future_time: HashMap<u8, Option<TrellisRoute>>,
    decoder_lookup: ConvolutionalDecoderLookup,
}

impl ViterbiTransfer {
    fn identify_best_source(&mut self, encoder_output: &u8, future_state: &u8, current_time: &HashMap<u8, Option<TrellisRoute>>) -> TrellisRoute { // doing it this way to minimize memory reallocations due to clones
        let mut best_source: u8 = 0;
        let mut best_distance: u8 = 254;
        for (current_state, transition) in self.decoder_lookup.state_transition(*future_state).iter() {
            dbg!("{} {} {}", &current_time, &current_state, &transition);
            match &current_time[current_state] {
                Some(route) => {
                    let distance = hamming_distance((*encoder_output, transition.output)) + route.hamming_distance; // precompute hamming distances

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

    pub fn update_routes(&mut self, encoder_output: &u8, current_time: HashMap<u8, Option<TrellisRoute>>) -> HashMap<u8, Option<TrellisRoute>> {
        let keys: Vec<u8> = self.future_time.keys().cloned().collect();

        for future_state in keys {
            dbg!("Stuff {}", &future_state);
            let new_route = Some(self.identify_best_source(encoder_output, &future_state, &current_time));
            self.future_time.insert(future_state, new_route);
        }

        let new_current_time = self.future_time.clone();
        self.future_time.clear();

        return new_current_time;
    }
}

pub struct ConvolutionalDecoder {
    output_factory: ConvolutionalOutputByteFactory,
    input_consumer: ConvolutionalInputConsumer,
    params: ConvolutionalParams,
    viterbi_transfer: ViterbiTransfer,
    blank_state_tree: HashMap<u8, Option<TrellisRoute>>
}

impl ConvolutionalDecoder {
    pub fn new(params: ConvolutionalParams, num_threads: usize) -> ConvolutionalDecoder { // make it so the lookup is generated in here with references and all, etc
        let decoding_lookup: ConvolutionalDecoderLookup = ConvolutionalLookupGenerator::generate_decoding_lookup(&params);
        let state_tree: HashMap<u8, Option<TrellisRoute>> = decoding_lookup.generate_state_vec()
            .into_iter()
            .map(|key| (key, None))
            .collect();

        ConvolutionalDecoder {
            params: params.clone(),
            output_factory: ConvolutionalOutputByteFactory::new(&params),
            input_consumer: ConvolutionalInputConsumer::new(Box::new(ViterbiProcessor{}), params.clone()), // processor is an asset of the consumer. Should ahve to go through consumer to access processor. So why does this struct maintain ownership? fix
            viterbi_transfer: ViterbiTransfer {future_time: state_tree.clone(), decoder_lookup: decoding_lookup},
            //parallelized_decoders: Vec::with_capacity(num_threads),
            blank_state_tree: state_tree
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

    fn get_best_route(&self, final_states: HashMap<u8, Option<TrellisRoute>>) -> TrellisRoute {
        let mut best_distance = 254;
        let mut best_route: Option<TrellisRoute> = None;

        for route in final_states.values() {
            match route {
                Some(valid_route) => {
                    if valid_route.hamming_distance < best_distance {
                        best_distance = valid_route.hamming_distance;
                        best_route = Some(valid_route.clone());
                    }
                }
                None => (),
            };
        };

        return best_route.unwrap();
    }

    fn viterbi(&mut self, input: Vec<u8>) -> Vec<u8> {
        let mut new_states: HashMap<u8, Option<TrellisRoute>> = self.blank_state_tree.clone();
        new_states.insert(0, Some(TrellisRoute {route: vec![TrellisStateChangeDecode {output: 0, input: 0}], hamming_distance: 0}));
        for output_bitstream in input.iter() {
            new_states = self.viterbi_transfer.update_routes(output_bitstream, new_states);
        };

        let mut final_route: TrellisRoute = self.get_best_route(new_states);
        final_route.route.remove(0); // must get rid of the initial dummy state. the 00 is not included in the output, necessarily
        return self.trellis_route_to_stream(&final_route);
    }
}

impl PipelineStep<Vec<u8>, Vec<u8>> for ConvolutionalDecoder {
    fn run(&mut self, input: Vec<u8>) -> Vec<u8> {
        let separated = self.input_consumer.consume(&input);
        self.viterbi(separated)
    }
}

