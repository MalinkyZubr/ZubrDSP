use super::window::*;


pub struct RectangularWindow {}
impl WindowFunction for RectangularWindow {
    fn window_function(&self, sample: f32, window_size: f32) -> f32 {
        return 1.0;
    }
}


pub struct TriangularWindow {}
impl WindowFunction for TriangularWindow {
    fn window_function(&self, sample: f32, window_size: f32) -> f32 {
        return 1.0 - ((sample - (window_size / 2.0)) / (window_size / 2.0)).abs();
    }
}


pub struct WelchWindow {}
impl WindowFunction for WelchWindow {
    fn window_function(&self, sample: f32, window_size: f32) -> f32 {
        let size_by_2 = window_size / 2.0;
        let inner_term = (sample - size_by_2) / size_by_2;
        return 1.0 - (inner_term).powf(2.0);
    }
}