use num::Complex;

pub trait ValidFloat {}
impl ValidFloat for f32 {}
impl ValidFloat for f64 {}

pub trait ValidComplex {}
impl<T: ValidFloat> ValidComplex for Complex<T> {}

pub trait ValidBytes {}
impl ValidBytes for u8 {}

pub trait ValidDSPNumerical {}
impl<T: ValidFloat> ValidDSPNumerical for Complex<T> {}
impl ValidDSPNumerical for f32 {}
impl ValidDSPNumerical for f64 {}