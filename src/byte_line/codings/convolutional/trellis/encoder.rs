use crate::pipeline::api::*;
use super::encoder_io::{ConvolutionalOutputByteFactory, ConvolutionalInputConsumer, ConvolutionalInputProcessor};
use super::trellis::{ConvolutionalEncoderLookup, ConvolutionalLookupGenerator};
use super::params::ConvolutionalParams;


struct ConvolutionalEncoderProcessor {
    state: u8,
    output_factory: ConvolutionalOutputByteFactory,
    encoding_lookup: ConvolutionalEncoderLookup,
}

impl ConvolutionalInputProcessor for ConvolutionalEncoderProcessor {
    fn process(&mut self, stream: u8) -> Option<u8> {
        let state_change = self.encoding_lookup.state_transition(self.state, stream);

        let result = self.output_factory.append(state_change.output);

        self.state = state_change.new_state;

        result
    }
}

pub struct ConvolutionalEncoder {
    consumer: ConvolutionalInputConsumer,
}

impl PipelineStep<Vec<u8>, Vec<u8>> for ConvolutionalEncoder {
    fn run(&mut self, input: ReceiveType<Vec<u8>>) -> Result<SendType<Vec<u8>>, String> {
        // Remove Option handling and use input directly
        match input {
            ReceiveType::Single(value) => Ok(SendType::NonInterleaved(self.consumer.consume(&value))),
            _ => Err(String::from("Cannot process multiple inputs"))
        }
    }
}

impl ConvolutionalEncoder {
    pub fn new(params: ConvolutionalParams) -> ConvolutionalEncoder {
        let byte_processor = ConvolutionalEncoderProcessor {
            state: 0,
            output_factory: ConvolutionalOutputByteFactory::new(&params),
            encoding_lookup: ConvolutionalLookupGenerator::generate_encoding_lookup(&params)
        };

        let consumer: ConvolutionalInputConsumer = ConvolutionalInputConsumer::new(
            Box::new(byte_processor),
            params.clone()
        );

        ConvolutionalEncoder {
            consumer
        }
    }
}