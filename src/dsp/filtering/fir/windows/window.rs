pub trait WindowFunction {
    fn window_function(&self, sample: f32, window_size: f32) -> f32;
}

pub fn apply_window<T: WindowFunction>(impulse_response: &mut Vec<f32>, window: T) {
    for (index, sample) in impulse_response.iter_mut().enumerate() {
        let impulse_response_size = impulse_response.len() as f32;
        *sample = *sample * window.window_function(index as f32, impulse_response_size as f32);
    }
}