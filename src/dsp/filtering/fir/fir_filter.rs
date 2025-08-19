use num::Complex;
use crate::dsp::fft::fftshift::fft_shift;
use crate::dsp::filtering::fir::ideal_response::fir_utils::{FIRFilter, generate_frequency_buffer};
use crate::dsp::filtering::fir::windows::window::{apply_window, WindowFunction};
use crate::dsp::system_response::system_functions::ImpulseResponse;
use crate::pipeline::api::*;


pub fn generate_FIR_impulse_response<W, F>(ideal_filter: F, window: W, buffer_size: usize, sample_rate: f32) -> ImpulseResponse 
where W: WindowFunction,
      F: FIRFilter,
{
    let transfer = ideal_filter.transfer_function(generate_frequency_buffer(buffer_size, sample_rate));
    let impulse_response = transfer.impulse_response(0);

    let mut impulse_response_function = impulse_response.reversed_impulse_response;
    impulse_response_function.reverse();
    let length = impulse_response_function.len();
    impulse_response_function.rotate_right(length / 2);

    let window_size = impulse_response_function.len();

    let windowed_response: Vec<f32> = impulse_response_function.iter()
        .enumerate()
        .map(|(n, x_n)| window.window_function(n as u32, window_size) * x_n)
        .collect();

    let new_impulse_response = ImpulseResponse::new_configured(windowed_response);

    new_impulse_response
}