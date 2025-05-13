use crate::Pipeline::node::prototype::PipelineStep;
use std::f32;


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PSKPoint {
    BPSK = 2,
    QPSK = 4,
    PSK8 = 8 // limiting to 8 because demodulation becomes hard and unpacking is also more difficult. Might add higher order PSK in future, but this is good for now
}

#[derive(Clone, PartialEq, Debug)]
pub struct PhaseVectorGenerator {
    point: PSKPoint,
    symbol_table: Vec<f32>, // only needs a vec since they are indexed in order anyway
    byte_unpack_window: u8,
    byte_unpack_window_size: u8
}

impl PhaseVectorGenerator {
    // maximum phase must be in RADIANS!
    fn generate_symbol_table(point: PSKPoint, maximum_phase: f32) -> Vec<f32> { // point corresponds to how many poles the psk has. you can encode log_2(n) symbols for n poles
        let mut lookup_table = Vec::with_capacity(point.clone() as usize);

        let mut current_symbol: f32 = 0.0;
        let phase_interval =  maximum_phase / (point as i32 as f32 - 1.0);

        while current_symbol < point as i32 as f32 {
            lookup_table.push(current_symbol * phase_interval);
            current_symbol += 1.0;
        }

        return lookup_table;
    }

    pub fn new(point: PSKPoint , maximum_phase: f32) -> PhaseVectorGenerator {// 1 symbol is 1 full period, can be caluclated
        let symbol_table = PhaseVectorGenerator::generate_symbol_table(point, maximum_phase);
        let table_size = symbol_table.len();

        PhaseVectorGenerator {
            point, 
            symbol_table,
            byte_unpack_window: point as u8 - 1, // 2^n - 1 for max value for given bits, 2^(log2n) = n bits in the mask, so simplifies to n - 1 as the value of the mask
            byte_unpack_window_size: (point as i32 as f32).log2() as u8
        }
    }

    fn unpack_byte(&self, mut byte: u8, phase_buffer: &mut Vec<f32>) {
        let mut byte_index: u8 = 0;

        while byte_index < (8 / (self.point as u8 / 2)) {
            let unpacked_baseband_datapoint: usize = (byte & self.byte_unpack_window) as usize;
            byte = byte >> self.byte_unpack_window_size;

            let acquired_phase: &f32 = self.symbol_table.get(unpacked_baseband_datapoint).unwrap();

            phase_buffer.push(acquired_phase.clone());

            byte_index += 1;
        }
    }

    fn generate_phase(&self, input: &Vec<u8>) -> Vec<f32> {
        let mut psk_buffer: Vec<f32> = Vec::with_capacity(((input.len() as u32) * (8 / self.byte_unpack_window_size) as u32) as usize);

        for data_byte in input.iter() {
            self.unpack_byte(data_byte.clone(), &mut psk_buffer);
        }

        dbg!("{}", &psk_buffer);

        return psk_buffer;
    }
}

impl PipelineStep<Vec<u8>, Vec<f32>> for PhaseVectorGenerator {
    fn run(&mut self, input: Vec<u8>) -> Vec<f32> {
        let result = self.generate_phase(&input);
        return result;
    }
}