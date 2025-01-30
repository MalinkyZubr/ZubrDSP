use super::params::ConvolutionalParams;

pub struct ConvolutionalOutputByteFactory {
    byte: u16,
    counter: u8,
    input_length: u8
}

impl ConvolutionalOutputByteFactory {
    pub fn new(input_length: u8) -> ConvolutionalOutputByteFactory {
        ConvolutionalOutputByteFactory {
            byte: 0,
            counter: 0,
            input_length
        }
    }

    pub fn append(&mut self, input: u8) -> Option<u8> { 
        let shifted_input: u16 = (input as u16) << self.counter;
        self.byte |= shifted_input;

        self.counter += self.input_length;

        if self.counter >= 8 {
            let return_val: Option<u8> = Some(self.byte as u8);
            self.byte >>= 8;
            self.counter = 0;
            return return_val;
        }
        else {
            return None;
        }
    }

    pub fn force_complete(&mut self) -> Option<u8> {
        if self.counter > 0 {
            let return_val: Option<u8> = Some(self.byte as u8);
            self.byte = 0;
            self.counter = 0;

            return return_val;
        }

        else {
            return None;
        }
    }
}

pub trait ConvolutionalInputProcessor {
    fn process(&mut self, stream: u8) -> Option<u8>;
}

pub struct ConvolutionalInputConsumer {
    processor: Box<dyn ConvolutionalInputProcessor + Send>,
    params: ConvolutionalParams
}

impl<T> ConvolutionalInputConsumer {
    pub fn new(processor: Box<dyn ConvolutionalInputProcessor + Send>, params: ConvolutionalParams) -> ConvolutionalInputConsumer<T> {
        ConvolutionalInputConsumer {
            processor,
            params
        }
    }  

    pub fn consume(&mut self, input: &Vec<u8>) -> Vec<u8> {
        let mut index: usize = 0;
        let length: usize = input.len();
        let output_size = length * self.params.output_polynomials.len() / 8;
        let mut output: Vec<T> = Vec::with_capacity(output_size);

        while index < length {
            let input_byte = input[index];
            let mut bit_count = 0;

            while bit_count < 8 { // turn this into a separate function. Lazy right now, do later
                let stream = (input_byte >> bit_count) & self.params.read_mask;

                match self.processor.process(stream) {
                    Some(result) => output.push(result),
                    None => (),
                };

                bit_count += self.params.input_bits;
            }

            index += 1;
        }

        output
    }
}