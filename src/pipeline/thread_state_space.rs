use num_enum::TryFromPrimitive;


#[repr(u8)]
#[derive(PartialEq, Debug, TryFromPrimitive, strum::Display, Clone)]
pub enum ThreadStateSpace {
    RUNNING = 0,
    PAUSED = 1,
    KILLED = 2
}