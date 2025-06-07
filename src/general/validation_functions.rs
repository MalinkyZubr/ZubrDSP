
pub fn is_power_2(num: i32) -> bool {
    return ((num - 1) & num) == 0;
}