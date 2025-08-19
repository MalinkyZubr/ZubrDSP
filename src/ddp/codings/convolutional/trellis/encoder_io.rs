use super::params::ConvolutionalParams;

pub struct ConvolutionalOutputByteFactory {
    byte: u16,
    counter: u8,
    input_length: u8
}

impl ConvolutionalOutputByteFactory { // update me to take params to make shifting only by 2's power
    pub fn new(params: &ConvolutionalParams) -> ConvolutionalOutputByteFactory {
        ConvolutionalOutputByteFactory {
            byte: 0,
            counter: 0,
            input_length: params.output_polynomials.len() as u8
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

impl ConvolutionalInputConsumer { // does this account for less than byte size contexts? or does it need to if these are packed?
    pub fn new(processor: Box<dyn ConvolutionalInputProcessor + Send>, params: ConvolutionalParams) -> ConvolutionalInputConsumer {
        ConvolutionalInputConsumer {
            processor,
            params
        }
    }  

    pub fn consume(&mut self, input: &Vec<u8>) -> Vec<u8> {
        let mut index: usize = 0;
        let length: usize = input.len();
        let output_size = length * self.params.output_polynomials.len() / 8;
        let mut output: Vec<u8> = Vec::with_capacity(output_size);

        while index < length {
            let input_byte = input[index];
            let mut bit_count = 0;

            while bit_count < 8 {
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