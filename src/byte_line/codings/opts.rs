pub fn check_parity(stream: &u8) -> u8 { // 1 is odd
    let mut parity: u8 = 0;
    let mut index: u8 = 0;

    while index < 8 {
        let bit_val: u8 = (*stream >> index) & 1;
        parity ^= bit_val;
        index += 1;
    }

    parity
}

pub fn hamming_distance(inputs: (u8, u8)) -> u8 {
    let xored: u8 = inputs.0 ^ inputs.1;
    let mut index: u8 = 0;
    let mut sum: u8 = 0;

    while index < 8 {
        sum += (xored >> index) & 1;
        index += 1;
    }

    sum
}