use super::pipeline_traits::Sharable;


pub enum CommType {
    Receiver,
    Sender
}


pub enum PipelineCommResult<T: Sharable> {
    CommError(CommType),
    Ok(T),
    Timeout,
    ResourceNotExist
}