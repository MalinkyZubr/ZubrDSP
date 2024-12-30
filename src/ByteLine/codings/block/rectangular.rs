use crate::Pipeline::node::prototype::PipelineStep;
use num::integer::Roots;


enum ParityEnum {
    EVEN,
    ODD
}

fn check_parity(stream: &[u8]) -> ParityEnum {

}

// remember, if a stream of several bytes is even parity, so are all the contained bits

pub fn RectangularEncode(input: Vec<u8>, parity: ParityEnum) -> Vec<u8> {
    let length = input.len() as u32;
    if((length).sqrt().pow(2) != length) {
        panic!("The unencoded data must have squarable dimension");
        Vec::new()
    }
    else {

    }
}