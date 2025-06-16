// deprecating the format adapters, switching to type adapters which is more essential
// format adapters were designed for a previous (invalid) understanding of how some DSP algorithms worked. They only create latency
// type adapters (primarily modulators) are much more important and baked directly into a proper DSP pipeline for radio transmission and reception
use super::node_enum;
use super::pipeline_thread::PipelineThread;

pub struct RadioPipeline {
    nodes: Vec<PipelineThread>
    pipeline_construction_queue: Vec<PipelineThread>,
}
impl RadioPipeline {
    pub fn new() -> RadioPipeline {RadioPipeline{nodes: Vec::new()}}
    pub fn add_step<I, O>(&mut self) {
        
    }
}