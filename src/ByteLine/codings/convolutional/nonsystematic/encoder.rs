use crate::Pipeline::node::prototype::PipelineStep;
use super::encoder_io::{ConvolutionalOutputByteFactory, ConvolutionalInputConsumer, ConvolutionalInputProcessor};
use super::trellis::ConvolutionalLookupTable;
use super::params::ConvolutionalParams;


struct ConvolutionalEncoderProcessor {
    context: u8,
    output_factory: ConvolutionalOutputByteFactory,
    encoding_lookup: ConvolutionalLookupTable,
}

impl ConvolutionalInputProcessor for ConvolutionalEncoderProcessor {
    fn process(&mut self, stream: u8) -> Option<u8> {
        let (new_context, encoded_output) = self.encoding_lookup.lookup[&self.context][&stream];
                
        let result = self.output_factory.append(encoded_output);

        self.context = new_context;

        result
    }
}

pub struct ConvolutionalEncoder {
    consumer: ConvolutionalInputConsumer,
}

impl PipelineStep<Vec<u8>> for ConvolutionalEncoder {
    fn run(&mut self, input: Vec<u8>) -> Vec<u8> {
        self.consumer.consume(&input)
    }
}

impl ConvolutionalEncoder {
    pub fn new(params: ConvolutionalParams) -> ConvolutionalEncoder {
        let byte_processor = ConvolutionalEncoderProcessor {
            context: 0,
            output_factory: ConvolutionalOutputByteFactory::new(params.input_bits),
            encoding_lookup: ConvolutionalLookupTable::new(params.clone())
        };

        let consumer: ConvolutionalInputConsumer<u8> = ConvolutionalInputConsumer::new(
            Box::new(byte_processor),
            params.clone()
        );

        ConvolutionalEncoder {
            consumer
        }
    }
}
