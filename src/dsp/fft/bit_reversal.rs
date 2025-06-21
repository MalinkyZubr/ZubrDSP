use num::Complex;
use std::f32::consts::{PI};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::general::parallel_computation::{self, *};
use rayon::{ThreadPool, ThreadPoolBuilder};
use crate::pipeline::prototype::PipelineStep;

//#[derive(Clone, Copy)]
enum ParityEnum {
    Even = 0,
    Odd = 1
}

// static twiddle computation is less time efficient, apparently. Cache really makes a difference
pub struct FFTBitReversal { // log_2(n) levels to the n sized fft for radix 2, nlogn total space in the vector
    bit_reversal_mapping: HashMap<usize, usize>,
    fft_size: usize,
    //twiddle_factors: HashMap<usize, Vec<Complex<f32>>>,
    index_bits_needed: usize,
    parallel_unit: ThreadPool,
    num_threads: usize,
    is_ifft: bool
}

impl FFTBitReversal {
    pub fn new(buffer_size: usize, num_threads: usize, is_ifft: bool) -> Self {
        let index_bits_needed = (buffer_size as f64).log2() as usize;

        let pool = ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap();
        
        FFTBitReversal { 
            bit_reversal_mapping: Self::generate_bit_reversal_mapping(buffer_size, index_bits_needed),
            fft_size: buffer_size,
            //twiddle_factors: Self::compute_twiddle_factors_all(buffer_size),
            index_bits_needed,
            parallel_unit: pool,
            num_threads,
            is_ifft
        }
    }

    fn compute_twiddle_factors_all(buffer_size: usize) -> HashMap<usize, Vec<Complex<f32>>> {
        let mut size: usize = 2;
        let mut twiddle_factors: HashMap<usize, Vec<Complex<f32>>> = HashMap::new();

        while size <= buffer_size {
            twiddle_factors.insert(size, Self::compute_twiddle_factors(size));
            size *= 2;
        }

        return twiddle_factors
    }

    fn compute_twiddle_factors(buffer_size: usize) -> Vec<Complex<f32>> {
        let mut twiddles: Vec<Complex<f32>> = Vec::with_capacity(buffer_size / 2);

        for index in 0..(buffer_size / 2) {
            let real_comp = (-2.0 * PI * index as f32 / buffer_size as f32).cos();
            let imag_comp = (-2.0 * PI * index as f32 / buffer_size as f32).sin();

            twiddles.push(Complex::new(real_comp, imag_comp));
        };

        return twiddles;
    }

    pub fn get_bit_reversal(value: usize, index_bits_needed: usize) -> usize { // no need for max efficiency since this only happens at the very beginning
        let string_size: usize = index_bits_needed;
        let mut reversed: usize = 0;

        for right_side_index in 0..(string_size / 2) {
            let left_side_index = string_size - 1 - right_side_index;
            let left_side_value = ((value & (1 << left_side_index)) > 0) as usize;
            let right_side_value = ((value & (1 << right_side_index)) > 0) as usize;

            reversed |= left_side_value << right_side_index;
            reversed |= right_side_value << left_side_index;
        }

        if string_size % 2 == 1 {
            reversed |= value & (1 << (string_size / 2));
        }

        return reversed;
    }

    fn generate_bit_reversal_mapping(buffer_size: usize, index_bits_needed: usize) -> HashMap<usize, usize> {
        let mut reversal_map: HashMap<usize, usize> = HashMap::new();

        for start_index in 0..buffer_size {
            let reversed_index = FFTBitReversal::get_bit_reversal(start_index, index_bits_needed);

            if reversed_index != start_index {
                match reversal_map.get(&reversed_index) {
                    Some(_value) => {},
                    None => {reversal_map.insert(start_index, reversed_index); ()}
                }
            }
        }

        return reversal_map;
    }

    fn bit_reversal_in_place(&self, buffer: &mut [Complex<f32>]) {
        for first_index in 0..buffer.len() {
            match self.bit_reversal_mapping.get(&first_index) {
                None => {},
                Some(second_index) => {
                    let temp = buffer[first_index];
                    buffer[first_index] = buffer[*second_index];
                    buffer[*second_index] = temp;
                }
            }
        }
    }

    fn get_proper_twiddle_factor(compute_index: usize, butterfly_size: usize) -> Complex<f32> {
        let angle = -2.0 * PI * compute_index as f32 / butterfly_size as f32;
        Complex::new(0.0, angle).exp()
    }

    fn compute_single_butterfly(&self, butterfly_size: usize, butterfly_index: usize, buffer: &mut[Complex<f32>]) {
        for compute_index in 0..(butterfly_size / 2) {
            let true_index = butterfly_index + compute_index;
            let true_offset_index = true_index + (butterfly_size / 2);

            let p = buffer[true_index];
            let q = buffer[true_offset_index] * //self.twiddle_factors[&butterfly_size][compute_index];
                Self::get_proper_twiddle_factor(compute_index, butterfly_size);

            buffer[true_index] = p + q;
            buffer[true_offset_index] = p - q;
        }
    }

    fn compute_fft(&self, buffer: &mut [Complex<f32>]) { // iterative version
        self.bit_reversal_in_place(buffer);

        for fft_stage in 1..self.index_bits_needed + 1 {
            let butterfly_size = 1 << fft_stage;
            let mut butterfly_index = 0;

            while butterfly_index < self.fft_size {
                self.compute_single_butterfly(butterfly_size, butterfly_index, buffer);
                
                butterfly_index += butterfly_size;
            }
        }
    }

    fn compute_single_butterfly_parallel(&self, butterfly_size: usize, slice: &mut[Complex<f32>]) {
        for even_index in 0..(butterfly_size / 2) {
            let odd_index = even_index + (butterfly_size / 2);

            let p = slice[even_index];
            let q = slice[odd_index] * //self.twiddle_factors[&butterfly_size][compute_index];
                Self::get_proper_twiddle_factor(even_index, butterfly_size);

            slice[even_index] = p + q;
            slice[odd_index] = p - q;
        }
    }

    fn compute_fft_parallel(&mut self, mut buffer: Vec<Complex<f32>>) -> Vec<Complex<f32>>{
        self.bit_reversal_in_place(&mut buffer);

        for fft_stage in 1..self.index_bits_needed + 1 {
            let butterfly_size = 1 << fft_stage;
            let chunks = buffer.chunks_mut(butterfly_size);
            
            self.parallel_unit.scope(
                |s| {
                    for chunk in chunks {
                        s.spawn(|s| {
                            self.compute_single_butterfly_parallel(butterfly_size, chunk);
                        })
                    }
                }

            )
        }

        return buffer;
    }

    pub fn fft(&mut self, mut buffer: Vec<Complex<f32>>) -> Vec<Complex<f32>> {
        if self.num_threads > 1 {
            self.compute_fft(&mut buffer);
            return buffer
        }
        else {
            return self.compute_fft_parallel(buffer);
        }
    }

    pub fn ifft(&mut self, mut buffer: Vec<Complex<f32>>) -> Vec<Complex<f32>> {
        for value in buffer.iter_mut() {
            *value = value.conj();
        }

        buffer = self.fft(buffer);

        let length = buffer.len();
        for value in buffer.iter_mut() {
            *value = *value / length as f32;
        }

        return buffer;
    }
}


impl PipelineStep<Vec<Complex<f32>>, Vec<Complex<f32>>> for FFTBitReversal {    
    fn run(&mut self, input: Option<Vec<Complex<f32>>>) -> Vec<Complex<f32>> {
        let mut input = input.unwrap();
        if self.is_ifft {
            return self.ifft(input);
        }
        else {
            return self.fft(input);
        }
    }   
}