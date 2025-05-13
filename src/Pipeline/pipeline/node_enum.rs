use std::fmt::Debug;

use crate::Pipeline::node::format_adapter::{ScalarToVectorAdapter, VectorToScalarAdapter};
use crate::Pipeline::node::prototype::{PipelineNode, PipelineNodeGeneric};


pub enum PipelineNodeEnum<T: Send + Clone + 'static + Debug> {
    Scalar(PipelineNode<T, T>),
    Vector(PipelineNode<Vec<T>, Vec<T>>),
    ScalarVectorAdapter(ScalarToVectorAdapter<T>),
    VectorScalarAdapter(VectorToScalarAdapter<T>)
}

impl<T: Send + Clone + 'static + Debug> PipelineNodeEnum<T> {
    // pub fn get_generalized(self) -> Arc<Mutex<dyn PipelineNodeGeneric + Send>> {
    //     match self {
    //         PipelineNodeEnum::Scalar(node) => Arc::new(Mutex::new(node)),
    //         PipelineNodeEnum::Vector(node) => Arc::new(Mutex::new(node)),
    //         PipelineNodeEnum::ScalarVectorAdapter(node) => Arc::new(Mutex::new(node)),
    //         PipelineNodeEnum::VectorScalarAdapter(node) => Arc::new(Mutex::new(node))
    //     }
    // }
    pub fn call(&mut self) {
        match self {
            PipelineNodeEnum::Scalar(node) => node.call(),
            PipelineNodeEnum::Vector(node) => node.call(),
            PipelineNodeEnum::ScalarVectorAdapter(node) => node.call(),
            PipelineNodeEnum::VectorScalarAdapter(node) => node.call()
        }
    }
}