use std::f32::consts::PI;
use super::window::*;


pub enum RaisedCosineType {
    Hann,
    Hamming
}


pub struct RaisedCosineWindow {
    cosine_coefficient: f32
}
impl RaisedCosineWindow {
    fn new(cosine_type: RaisedCosineType) -> Self {
        let cosine_coefficient = 
            match cosine_type {
                RaisedCosineType::Hann => 0.5,
                RaisedCosineType::Hamming => 25.0 / 46.0
            };

        return RaisedCosineWindow {cosine_coefficient};
    }
}
impl WindowFunction for RaisedCosineWindow {
    fn window_function(&self, sample: f32, window_size: f32) -> f32 {
        let cosine_term = ((2.0 * PI * sample) / window_size).cos();
        return self.cosine_coefficient - ((1.0 - self.cosine_coefficient) * cosine_term);
    }
}


pub enum CosineSumType {
    Blackman,
    Nuttall,
    BlackmanNuttall,
    BlackmanHarris,
    Flattop,
    SecondPower,
    FourthPower,
    SixthPower,
    EighthPower
}


pub struct CosineSumWindow {
    coefficients: Vec<f32>,
}
impl CosineSumWindow {
    pub fn new(cosine_sum_type: CosineSumType) -> Self {
        let coefficients: Vec<f32> = match cosine_sum_type {
            CosineSumType::Blackman => vec![0.42, 0.5, 0.08],
            CosineSumType::Nuttall => vec![0.355768, 0.487396, 0.144232, 0.012604],
            CosineSumType::BlackmanNuttall => vec![0.3635819, 0.4891775, 0.1365995, 0.0106411],
            CosineSumType::BlackmanHarris => vec![0.35875, 0.48829, 0.14128, 0.01168],
            CosineSumType::Flattop => vec![0.21557895, 0.41663158, 0.277263158, 0.083578947, 0.006947368],
            CosineSumType::SecondPower => vec![0.5, 0.5],
            CosineSumType::FourthPower => vec![0.375, 0.5, 0.125],
            CosineSumType::SixthPower => vec![0.3125, 0.46875, 0.1875, 0.03125],
            CosineSumType::EighthPower => vec![0.2734375, 0.4375, 0.21875, 0.0625, 7.8125E-3]
        };

        return CosineSumWindow { coefficients };
    }
}
impl WindowFunction for CosineSumWindow {
    fn window_function(&self, sample: f32, window_size: f32) -> f32 {
        let mut sample_result = self.coefficients[0];
        let mut subtract = true;

        for (index, coefficient) in self.coefficients.iter().enumerate().skip(0) {
            let term = coefficient * ((2.0 * index as f32 * PI * sample) / window_size).cos();
            if subtract {
                sample_result -= term;
                subtract = false;
            }
            else {
                sample_result += term;
                subtract = true;
            }
        }

        return sample_result;
    }
}


pub struct SineWindow {}
impl WindowFunction for SineWindow {
    fn window_function(&self, sample: f32, window_size: f32) -> f32 {
        return (PI * sample / window_size).sin();
    }
}


