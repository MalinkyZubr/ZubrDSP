use crate::Pipeline::node::prototype::PipelineNode;
use crate::Pipeline::node::messages::create_node_connection;
use crate::Pipeline::node::buffer::{ScalarToVectorAdapter, VectorToScalarAdapter};
use super::node_enum::PipelineNodeEnum;


pub struct Welder {
    pub buff_size: usize
}

impl Welder {
    pub fn new(buff_size: usize) -> Welder {
        Welder {
            buff_size
        }
    } // DANGER: Implications of static is that it lives as long as the program in HEAP! May need explicit drop to prevent runtime leaks!
    pub fn weld<T: Clone + Send + 'static>(&self, src: &mut PipelineNodeEnum<T>, dst: &mut PipelineNodeEnum<T>) -> Option<PipelineNodeEnum<T>> {
        match (src, dst) {
            (PipelineNodeEnum::Scalar(src), 
            PipelineNodeEnum::Scalar(dst)) => {self.weld_scalar::<T>(src, dst); None},

            (PipelineNodeEnum::Vector(src), 
            PipelineNodeEnum::Vector(dst)) => {self.weld_vector::<T>(src, dst); None},

            (PipelineNodeEnum::Scalar(src), 
            PipelineNodeEnum::Vector(dst)) => Some(PipelineNodeEnum::ScalarVectorAdapter(self.weld_scalar_to_vector::<T>(src, dst))),

            (PipelineNodeEnum::Vector(src), 
            PipelineNodeEnum::Scalar(dst)) => Some(PipelineNodeEnum::VectorScalarAdapter(self.weld_vector_to_scalar::<T>(src, dst))),

            (_, _) => None
        }
    }

    fn weld_scalar<T: Clone + Send + 'static>(&self, src: &mut PipelineNode<T>, dst: &mut PipelineNode<T>) {
        let (sender, receiver) = create_node_connection::<T>();
        src.set_output(Box::new(sender));
        dst.set_input(Box::new(receiver));
    }

    fn weld_vector<T: Clone + Send + 'static>(&self, src: &mut PipelineNode<Vec<T>>, dst: &mut PipelineNode<Vec<T>>) {
        let (sender, receiver) = create_node_connection::<Vec<T>>();
        src.set_output(Box::new(sender));
        dst.set_input(Box::new(receiver));
    }

    fn weld_scalar_to_vector<T: Clone + Send + 'static>(&self, src: &mut PipelineNode<T>, dst: &mut PipelineNode<Vec<T>>) -> ScalarToVectorAdapter<T> {
        let (scalar_sender, scalar_receiver) = create_node_connection::<T>();
        let (vector_sender, vector_receiver) = create_node_connection::<Vec<T>>();
        
        src.set_output(Box::new(scalar_sender));
        let adapter: ScalarToVectorAdapter<T> = ScalarToVectorAdapter::new(scalar_receiver, vector_sender, self.buff_size);
        dst.set_input(Box::new(vector_receiver));

        return adapter;
    }

    fn weld_vector_to_scalar<T: Clone + Send + 'static>(&self, src: &mut PipelineNode<Vec<T>>, dst: &mut PipelineNode<T>) -> VectorToScalarAdapter<T> {
        let (scalar_sender, scalar_receiver) = create_node_connection::<T>();
        let (vector_sender, vector_receiver) = create_node_connection::<Vec<T>>();
        
        src.set_output(Box::new(vector_sender));
        let adapter: VectorToScalarAdapter<T> = VectorToScalarAdapter::new(vector_receiver, scalar_sender, self.buff_size);
        dst.set_input(Box::new(scalar_receiver));

        return adapter;
    }
}