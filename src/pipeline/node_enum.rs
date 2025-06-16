use std::fmt::{Debug, Pointer};
use std::sync::mpsc::{Sender, Receiver, channel};
use super::prototype::{PipelineNode, PipelineStep};
use num::Complex;
use std::sync::{Arc, Mutex};


#[allow(non_camel_case_types)]
pub trait HoldsF32_ComplexF32 {
    fn construct_node(&self, node: PipelineNode<Vec<f32>, Vec<Complex<f32>>>) -> Self;
}

#[allow(non_camel_case_types)]
pub trait HoldsComplexF32_F32 {
    fn construct_node(&self, node: PipelineNode<Vec<Complex<f32>>, Vec<f32>>) -> Self;
}

#[allow(non_camel_case_types)]
pub trait HoldsF32_F32 {
    fn construct_node(&self, node: PipelineNode<Vec<f32>, Vec<f32>>) -> Self;
}

#[allow(non_camel_case_types)]
pub trait HoldsComplexF32_ComplexF32 {
    fn construct_node(&self, node: PipelineNode<Vec<Complex<f32>>, Vec<Complex<f32>>>) -> Self;
}

#[allow(non_camel_case_types)]
pub trait HoldsByte_F32 {
    fn construct_node(&self, node: PipelineNode<Vec<u8>, Vec<f32>>) -> Self;
}

#[allow(non_camel_case_types)]
pub trait HoldsF32_Byte {
    fn construct_node(&self, node: PipelineNode<Vec<f32>, Vec<u8>>) -> Self;
}

#[allow(non_camel_case_types)]
pub trait HoldsByte_Byte {
    fn construct_node(&self, node: PipelineNode<Vec<u8>, Vec<u8>>) -> Self;
}

#[allow(non_camel_case_types)] // dont want to use the vtable for this. Enum is the only other option
pub enum PipelineStepEnum {
    F32_ComplexF32(PipelineNode<Vec<f32>, Vec<Complex<f32>>>),
    ComplexF32_F32(PipelineNode<Vec<Complex<f32>>, Vec<f32>>),
    Byte_F32(PipelineNode<Vec<u8>, Vec<f32>>),
    F32_Byte(PipelineNode<Vec<f32>, Vec<u8>>),
    F32_F32(PipelineNode<Vec<f32>, Vec<f32>>),
    ComplexF32_ComplexF32(PipelineNode<Vec<Complex<f32>>, Vec<Complex<f32>>>),
    Byte_Byte(PipelineNode<Vec<u8>, Vec<u8>>),
}

impl PipelineStepEnum {
    pub fn attach(this_wrapped: &mut Arc<Mutex<Self>>, other_wrapped: &mut Arc<Mutex<Self>>) {
        let mut this = this_wrapped.lock().unwrap();
        let mut other = other_wrapped.lock().unwrap();
        
        match this {
            PipelineStepEnum::F32_ComplexF32(mut pred) => {
                match other {
                    PipelineStepEnum::ComplexF32_ComplexF32(mut succ) => pred.attach(&mut succ),
                    PipelineStepEnum::ComplexF32_F32(mut succ) => pred.attach(&mut succ),
                    _ => panic!("Attach called on a non-complex pipeline step")
                }
            },
            PipelineStepEnum::ComplexF32_F32(mut pred) => {
                match other {
                    PipelineStepEnum::F32_ComplexF32(mut succ) => pred.attach(&mut succ),
                    PipelineStepEnum::F32_Byte(mut succ) => pred.attach(&mut succ),
                    PipelineStepEnum::F32_F32(mut succ) => pred.attach(&mut succ),
                    _ => panic!("Attach called on a non-float pipeline step")
                }
            }
            PipelineStepEnum::Byte_F32(mut pred) => {
                match other {
                    PipelineStepEnum::F32_Byte(mut succ) => pred.attach(&mut succ),
                    PipelineStepEnum::F32_F32(mut succ) => pred.attach(&mut succ),
                    PipelineStepEnum::F32_ComplexF32(mut succ) => pred.attach(&mut succ),
                    _ => panic!("Attach called on a non-byte pipeline step")
                }
            },
            PipelineStepEnum::F32_Byte(mut pred) => {
                match other {
                    PipelineStepEnum::Byte_F32(mut succ) => pred.attach(&mut succ),
                    PipelineStepEnum::Byte_Byte(mut succ) => pred.attach(&mut succ),
                    _ => panic!("Attach called on a non-byte pipeline step")
                }
            },
            PipelineStepEnum::F32_F32(mut pred) => { 
                match other {
                    PipelineStepEnum::F32_Byte(mut succ) => pred.attach(&mut succ),
                    PipelineStepEnum::F32_F32(mut succ) => pred.attach(&mut succ),
                    PipelineStepEnum::F32_ComplexF32(mut succ) => pred.attach(&mut succ),
                    _ => panic!("Attach called on a non-float pipeline step")
                }
            },
            PipelineStepEnum::ComplexF32_ComplexF32(mut pred) => { 
                match other {
                    PipelineStepEnum::ComplexF32_ComplexF32(mut succ) => pred.attach(&mut succ),
                    PipelineStepEnum::ComplexF32_F32(mut succ) => pred.attach(&mut succ),
                    _ => panic!("Attach called on a non-complex pipeline step")
                }
            },
            PipelineStepEnum::Byte_Byte(mut pred) => { 
                match other {
                    PipelineStepEnum::Byte_Byte(mut succ) => pred.attach(&mut succ),
                    PipelineStepEnum::Byte_F32(mut succ) => pred.attach(&mut succ),
                    _ => panic!("Attach called on a non-byte pipeline step")
                }
            }
        }
    }
}

impl HoldsF32_ComplexF32 for PipelineStepEnum {
    fn construct_node(&self, step: PipelineNode<Vec<f32>, Vec<Complex<f32>>>) -> Self {
        return PipelineStepEnum::F32_ComplexF32(step);
    }
}
impl HoldsComplexF32_F32 for PipelineStepEnum {
    fn construct_node(&self, step: PipelineNode<Vec<Complex<f32>>, Vec<f32>>) -> Self {
        return PipelineStepEnum::ComplexF32_F32(step);
    }
}
impl HoldsF32_F32 for PipelineStepEnum {
    fn construct_node(&self, step: PipelineNode<Vec<f32>, Vec<f32>>) -> Self {
        return PipelineStepEnum::F32_F32(step);
    }
}
impl HoldsComplexF32_ComplexF32 for PipelineStepEnum {
    fn construct_node(&self, step: PipelineNode<Vec<Complex<f32>>, Vec<Complex<f32>>>) -> Self {
        return PipelineStepEnum::ComplexF32_ComplexF32(step);
    }
}
impl HoldsByte_Byte for PipelineStepEnum {
    fn construct_node(&self, step: PipelineNode<Vec<u8>, Vec<u8>>) -> Self {
        return PipelineStepEnum::Byte_Byte(step);
    }
}
impl HoldsF32_Byte for PipelineStepEnum {
    fn construct_node(&self, step: PipelineNode<Vec<f32>, Vec<u8>>) -> Self {
        return PipelineStepEnum::F32_Byte(step);
    }
}
impl HoldsByte_F32 for PipelineStepEnum {
    fn construct_node(&self, step: PipelineNode<Vec<u8>, Vec<f32>>) -> Self {
        return PipelineStepEnum::Byte_F32(step);
    }
}