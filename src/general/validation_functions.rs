use num::{Num, Signed};

pub fn is_power_2(num: i32) -> bool {
    return ((num - 1) & num) == 0;
}

pub fn percent_error<T: Num + Signed>(observed: T, expected: T) -> f32 {
    let expected_clone = expected.clone();
    return ((observed as f32 - (expected as f32)) / expected_clone as f32).abs() * 100.0;
}