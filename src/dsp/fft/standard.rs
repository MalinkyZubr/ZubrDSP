use num::{complex::ComplexFloat, Complex};
use core::slice;
use std::f32::consts::{E, PI};
use num::traits::Pow;
use std::mem;
use crate::general::validation_functions::{self, is_power_2};
use crate::pipeline::node::prototype::PipelineStep;


// why not use in place FFT?
// lots of memory on a PC, simpler to see what happening maintaing traditional structure (even if static)
// easier to parallelize easily, strong decoupling of different stages
// all these above characteristics make it better for me (I think) than the usual in place FFT algorithms

// convert all of this to a flat buffer that is mutably sliced. Shared memory region with heap style access

#[derive(Clone, Copy)]
enum ParityEnum {
    Even = 0,
    Odd = 1
}

fn generate_input_references<'a>(raw_input: &'a Vec<f32>, input_indicies: &Vec<usize>, parity: ParityEnum) -> (Vec<&'a f32>, Vec<usize>) {
    let mapped_size = raw_input.len() / 2;
    let mut indicies: Vec<usize> = Vec::with_capacity(mapped_size);
    let mut mapped_input_vector: Vec<&f32> = Vec::with_capacity(mapped_size);

    for (index, (true_index, element)) in input_indicies.iter().zip(raw_input).enumerate() {
        match parity {
            ParityEnum::Even => {
                if index % 2 == 0 {
                    mapped_input_vector.push(element);
                    indicies.push(*true_index);
                }
            },
            ParityEnum::Odd => {
                if index % 2 == 1 {
                    mapped_input_vector.push(element);
                    indicies.push(*true_index);
                }
            }
        };
    };

    return (mapped_input_vector, indicies);
}

struct FFTOperation<'a> {
    twiddles: Vec<Complex<f32>>,
    buffer_size: usize,
    radix: u8,

    even_fft: Option<Box<FFTOperation<'a>>>, // base case decision is baked into the static computation tree at runtime start, no additional costs incurred
    odd_fft: Option<Box<FFTOperation<'a>>>,

    pub compute_vector: Vec<Complex<f32>>,
    input_vector: Vec<&'a f32>,

    //mem_buffer: FFTMemoryBuffer<'a>,
}

impl<'a> FFTOperation<'a> {
    pub fn new(buffer_size: usize, radix: u8, raw_input_vector: &'a Vec<f32>, reference_input_vector: Vec<&'a f32>, input_indicies: &Vec<usize>) -> Self {
        assert!(is_power_2(buffer_size as i32));
        assert!(is_power_2(radix as i32));

        let compute_vector = vec![Complex::<f32>::new(0.0, 0.0); reference_input_vector.len()];

        let mut this: FFTOperation<'a> = FFTOperation {
            twiddles: FFTOperation::compute_twiddle_factors(buffer_size),
            buffer_size: buffer_size,
            radix: radix,
            even_fft: None,
            odd_fft: None,
            input_vector: reference_input_vector,
            compute_vector
        };

        this.generate_children(raw_input_vector, input_indicies);

        return this;
    }

    fn generate_child(&mut self, raw_input: &'a Vec<f32>, input_indicies: &Vec<usize>, parity: ParityEnum) -> FFTOperation<'a> {
        let (input_vector, input_true_indicies): (Vec<&f32>, Vec<usize>) = generate_input_references(raw_input, input_indicies, parity);

        return FFTOperation::new(
                self.buffer_size / 2, self.radix,
                raw_input,
                input_vector,
                &input_true_indicies
            );
        
    }

    fn generate_children(&mut self, raw_input: &'a Vec<f32>, input_indicies: &Vec<usize>) {
        if self.buffer_size > self.radix as usize {
            self.even_fft = Some(Box::new(self.generate_child(raw_input, input_indicies, ParityEnum::Even)));
            self.odd_fft = Some(Box::new(self.generate_child(raw_input, input_indicies, ParityEnum::Odd)));
        }
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

    fn swap_compute_buffers(&mut self, parity: ParityEnum) {
        let vector_length = self.compute_vector.len();

        let child: &mut Box<FFTOperation<'a>> = match parity {
            ParityEnum::Even => self.even_fft.as_mut().unwrap(),
            ParityEnum::Odd => self.odd_fft.as_mut().unwrap()
        };

        for (index, buffer_value) in child.compute_vector.iter_mut().enumerate() {
            let insert_index = index + ((vector_length / 2) * parity as usize);
            mem::swap(buffer_value, self.compute_vector.get_mut(insert_index).unwrap());
        }
    }

    pub fn fft_standard(&mut self) {
        if self.even_fft.is_some() && self.odd_fft.is_some() {
            self.even_fft.as_mut().unwrap().fft_standard();
            self.odd_fft.as_mut().unwrap().fft_standard();
            self.swap_compute_buffers(ParityEnum::Even);
            self.swap_compute_buffers(ParityEnum::Odd);

            //let odd_buffer = odd_fft.as_mut().unwrap().fft_standard();
        }
        else {
            self.compute_vector[0] = Complex::new(*self.input_vector[0], 0.0);
            self.compute_vector[1] = Complex::new(*self.input_vector[1], 0.0);
        }

        for index in 0..self.compute_vector.len() / 2 {
            let p = self.compute_vector[index];
            let q = self.twiddles[index] * self.compute_vector[index + self.buffer_size / 2];

            self.compute_vector[index] = p + q;
            self.compute_vector[index + self.buffer_size / 2] = p - q;
        }
    }
}


pub enum FFTDirection {
    IFFT,
    FFT
}

fn ifft_precompute(input: &mut Vec<f32>) {

}

pub struct FFTTop {
    precomputation_function: fn(&mut Vec<f32>),
    postcomputation_function: fn(&mut Vec<Complex<f32>>),
    //operator_core: FFTOperation<'a>
}

impl PipelineStep<Vec<Complex<f32>>, Vec<Complex<f32>>> for FFTTop {

}

