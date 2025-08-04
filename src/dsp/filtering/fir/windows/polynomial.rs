use super::window::*;


pub struct RectangularWindow {}
impl WindowFunction for RectangularWindow {
    fn window_function(&self, sample: u32, window_size: usize) -> f32 {
        return 1.0;
    }
}


pub struct TriangularWindow {}
impl WindowFunction for TriangularWindow {
    fn window_function(&self, sample: u32, window_size: usize) -> f32 {
        return 1.0 - ((sample as f32 - (window_size as f32 / 2.0)) / (window_size as f32 / 2.0)).abs();
    }
}


pub struct WelchWindow {}
impl WindowFunction for WelchWindow {
    fn window_function(&self, sample: u32, window_size: usize) -> f32 {
        let size_by_2 = window_size as f32 / 2.0;
        let inner_term = (sample as f32 - size_by_2) / size_by_2;
        return 1.0 - (inner_term).powf(2.0);
    }
}