use crate::dsp::filtering::{shared::filter::*};


pub struct ButterworthLowPass {
    filter: FilterParameters,
    pass_frequency_w: f32,
    max_attenuation: f32, // linear scale
    stop_frequency_w: f32,
    min_attenuation: f32 
}


impl Filter for ButterworthLowPass {
    fn find_order(&mut self) -> f32 {
        let min_atten_numerator = (10.0 as f32).powf(0.1 * self.min_attenuation) - 1.0;
        let max_atten_numerator = (10.0 as f32).powf(0.1 * self.max_attenuation) - 1.0;

        let numerator = (min_atten_numerator / max_atten_numerator).log10();
        let denominator = 2.0 * (self.stop_frequency_w / self.pass_frequency_w).log10();

        return numerator / denominator;
    }
    fn find_gain(&mut self) -> f32 {
        return 0.0;
    }
}