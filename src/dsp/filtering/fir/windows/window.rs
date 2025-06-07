pub trait WindowFunction {
    fn window_function(&self, sample: f32, window_size: f32) -> f32;
}

pub fn apply_window<T: WindowFunction>(impulse_response: Vec<f32>, window: T) -> Vec<f32> { // apply window to ideal impulse response in time domain
    let mut windowed_samples = Vec::new();

    for (index, sample) in impulse_response.iter().enumerate() {
        let impulse_response_size = impulse_response.len() as f32;
        windowed_samples.push(sample * window.window_function(index as f32, impulse_response_size as f32));
    }

    return windowed_samples;
}