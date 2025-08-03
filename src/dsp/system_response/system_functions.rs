use rustfft::FftPlanner;
use num::Complex;

pub struct ImpulseResponse {
    pub reversed_impulse_response: Vec<f32>,
}
impl ImpulseResponse {
    pub fn new_configured(mut impulse_response: Vec<f32>) -> Self {
        impulse_response.reverse();
        let ir_len = impulse_response.len();

        ImpulseResponse {
            reversed_impulse_response: impulse_response,
        }
    }
    pub fn new_from_function<F>(func: F, sample_start_n: i64, window_size: usize) -> Self
    where F: Fn(i64) -> f32 {
        let mut impulse_response = Vec::with_capacity(window_size);
        for index in sample_start_n..sample_start_n + window_size as i64 {
            impulse_response.push(func(index));
        }

        ImpulseResponse {
            reversed_impulse_response: impulse_response,
        }
    }

    pub fn transfer_function(mut self, padding: usize) -> TransferFunction {
        self.reversed_impulse_response.reverse();
        let mut impulse_response = self.reversed_impulse_response;

        if padding > 0 {
            impulse_response.extend(vec![0.0; padding]);
        }

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(impulse_response.len());
        let mut complex_impulse_response: Vec<Complex<f32>> = impulse_response
            .iter_mut()
            .map(|x| Complex::new(*x, 0.0))
            .collect();

        fft.process(&mut complex_impulse_response);
            
        TransferFunction::new_configured(
            complex_impulse_response,
        )
    }
    
    pub fn len(&self) -> usize {
        return self.reversed_impulse_response.len();
    }
}

pub struct TransferFunction {
    pub transfer_function: Vec<Complex<f32>>
}
impl TransferFunction {
    pub fn new_configured(transfer_function: Vec<Complex<f32>>) -> Self {
        TransferFunction {
            transfer_function,
        }
    }
    pub fn new_from_function<F>(func: F, sample_start_n: i64, window_size: usize) -> Self
    where F: Fn(i64) -> Complex<f32> {
        let mut transfer_function = Vec::with_capacity(window_size);
        for index in sample_start_n..sample_start_n + window_size as i64 {
            transfer_function.push(func(index));
        }
        
        TransferFunction {
            transfer_function
        }
    }

    pub fn impulse_response(mut self, dropped: usize) -> ImpulseResponse {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_inverse(self.transfer_function.len());
        
        fft.process(&mut self.transfer_function);
        
        self.transfer_function.truncate(self.transfer_function.len() - dropped);

        let mut impulse_response: Vec<f32> = self.transfer_function
            .iter_mut()
            .map(|x| x.re)
            .collect();
        
        ImpulseResponse::new_configured(impulse_response)
    }

    pub fn len(&self) -> usize {
        return self.transfer_function.len();
    }
}