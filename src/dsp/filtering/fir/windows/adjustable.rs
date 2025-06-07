use std::f32::consts::PI;

use super::window::*;


pub struct GaussianWindow {
    sigma: f32
}
impl GaussianWindow {
    pub fn new(sigma: f32) -> Self {
        assert!(sigma <= 0.5);
        GaussianWindow {sigma}
    }
}
impl WindowFunction for GaussianWindow {
    fn window_function(&self, sample: f32, window_size: f32) -> f32 {
        let half_window = window_size / 2.0;
        let square_term = ((sample - half_window) / (self.sigma * half_window)).powf(2.0);

        return (-0.5 * square_term).exp();
    }
}


pub struct ConfinedGaussianWindow {
    sigma: f32
}
impl ConfinedGaussianWindow {
    pub fn new(sigma: f32) -> Self {
        assert!(sigma < 0.14);
        ConfinedGaussianWindow { sigma }
    }

    fn gaussian_function(&self, input: f32, window_size: f32) -> f32 {
        let const_l = window_size + 1.0;
        return (-((input - (window_size / 2.0)) / (2.0 * const_l * self.sigma)).powf(2.0)).exp()
    }
}
impl WindowFunction for ConfinedGaussianWindow {
    fn window_function(&self, sample: f32, window_size: f32) -> f32 {
        let numerator = self.gaussian_function(-0.5, window_size) * (
            self.gaussian_function(sample + window_size + 1.0, window_size) +
            self.gaussian_function(sample - window_size - 1.0, window_size)
        );

        let denominator = self.gaussian_function(-0.5 + window_size + 1.0, window_size) +
            self.gaussian_function(-0.5 - window_size - 1.0, window_size);

        return numerator / denominator;
    }
}


pub struct TukeyWindow {
    alpha: f32
}
impl TukeyWindow {
    pub fn new(alpha: f32) -> Self {
        assert!(alpha <= 1.0 && alpha >= 0.0);
        TukeyWindow{alpha}
    }
}
impl WindowFunction for TukeyWindow { // remember the rule on wikipedia! this isnt the full story!
    fn window_function(&self, sample: f32, window_size: f32) -> f32 {
        if 0.0 <= sample && sample < (self.alpha * window_size / 2.0) {
            return 0.5 * (1.0 - (2.0 * PI * sample / (self.alpha * window_size)));
        }
        else if (self.alpha * window_size / 2.0) <= sample && sample <= window_size / 2.0 {
            return 1.0;
        }
        else {
            return 0.5 * (1.0 - (2.0 * PI * sample / (self.alpha * window_size)));
        }
    }
}


