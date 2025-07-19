// deprecating the format adapters, switching to type adapters which is more essential
// format adapters were designed for a previous (invalid) understanding of how some DSP algorithms worked. They only create latency
// type adapters (primarily modulators) are much more important and baked directly into a proper DSP pipeline for radio transmission and reception

use super::pipeline_thread::PipelineThread;
//use super::dummy::{dummy_thread_function, DummyManager, DummyRunner};
use std::thread::{self, JoinHandle};
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;


pub struct RadioPipeline {
    pub nodes: Vec<PipelineThread>,
    //dummy_manager: DummyManager
    pub retries: usize,
    pub timeout: u64,
    pub backpressure_val: usize
}
impl RadioPipeline {
    pub fn new(retries: usize, timeout: u64, backpressure_val: usize) -> RadioPipeline {
        RadioPipeline{
            nodes: Vec::new(), 
            retries, timeout, backpressure_val
            //dummy_manager: DummyManager::new()
        }
    }
    
    pub fn start(&mut self) {
        //self.dummy_manager.start();
        for node in self.nodes.iter_mut() {
            node.start();
        }
    }
    
    pub fn stop(&mut self) {
        //self.dummy_manager.stop();
        for node in self.nodes.iter_mut() {
            node.stop();
        }
    }
    
    pub fn kill(&mut self) {
        for mut node in self.nodes.iter_mut() {
            node.kill();
        }
    }
}