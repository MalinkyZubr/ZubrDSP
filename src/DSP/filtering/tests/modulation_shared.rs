use num::abs;

pub fn is_within_error_bounds(limit_error: f32, actual: f32, expected: f32) -> bool {
    let percent_error = abs(((actual - expected) / expected)) * 100.0;

    return percent_error < limit_error;
}


#[cfg(test)]
pub mod PSKTests {
    use crate::DSP::filtering::{filter::*, tests::modulation_shared::is_within_error_bounds};
    use std::f32::consts::PI;

    #[test]
    fn second_order_lowpass_digitizer() {
        let (numerator, denominator) = compute_z_coefficients_o2([0.0,0.0,1.0], [1.0, ((2.0 as f32).sqrt()), 1.0], 1.0);

        let ideal_numerator = [(1.0 / 1058.546), (2.0 / 1058.546), (1.0 / 1058.546)];
        let ideal_denominator = [1.0, (2023.090 / 1058.546), (-968.544) / 1058.546];

        dbg!("{} should be {}\n{} should be {}\n", &numerator, &ideal_numerator, &denominator, &ideal_denominator);

        for index in 0..numerator.len() {
            assert!(is_within_error_bounds(5.0, numerator[index], ideal_numerator[index]));
            assert!(is_within_error_bounds(5.0, denominator[index], ideal_denominator[index]));
        }
    }

    #[test]
    fn second_order_digitizer() {
        
    }
}