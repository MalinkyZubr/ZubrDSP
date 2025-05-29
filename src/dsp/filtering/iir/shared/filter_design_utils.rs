pub fn linear_to_db(gain_value: f32) -> f32 {
    return 20.0 * gain_value.log10();
}

pub fn db_to_linear(gain_value: f32) -> f32 {
    return (10.0 as f32).powf(gain_value / 20.0);
}