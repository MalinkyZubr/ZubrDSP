// deprecating the format adapters, switching to type adapters which is more essential
// format adapters were designed for a previous (invalid) understanding of how some DSP algorithms worked. They only create latency
// type adapters (primarily modulators) are much more important and baked directly into a proper DSP pipeline for radio transmission and reception

use super::pipeline_thread::PipelineThread;

pub struct RadioPipeline {
    pub nodes: Vec<PipelineThread>
}
impl RadioPipeline {
    pub fn new() -> RadioPipeline {RadioPipeline{nodes: Vec::new()}}
    pub fn start(&mut self) {
        for node in self.nodes.iter_mut() {
            node.start();
        }
    }
    
    pub fn stop(&mut self) {
        for node in self.nodes.iter_mut() {
            node.stop();
        }
    }
    
    pub fn kill(&mut self) {
        for node in self.nodes.iter_mut() {
            node.kill();
        }
    }
}