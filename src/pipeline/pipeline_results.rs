use super::pipeline_traits::Sharable;


#[derive(Debug, Clone, Copy)]
pub enum CommType {
    Receiver,
    Sender
}


#[derive(Debug, Clone, Copy)]
pub enum PipelineCommResult<T: Sharable> {
    CommError(CommType),
    Ok(T),
    Timeout,
    ResourceNotExist,
    IllegalDirective
}
impl<T: Sharable> PipelineCommResult<T> {
    pub fn unwrap(self) -> T {
        match self {
            PipelineCommResult::Ok(t) => t,
            _ => panic!("Could not unwrap PipelineCommResult")
        }
    }
    pub fn get_ok(&mut self) -> &mut T {
        match self {
            PipelineCommResult::Ok(t) => t,
            _ => panic!("Could not get PipelineCommResult")
        }
    }
}