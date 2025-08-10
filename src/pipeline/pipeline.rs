// deprecating the format adapters, switching to type adapters which is more essential
// format adapters were designed for a previous (invalid) understanding of how some DSP algorithms worked. They only create latency
// type adapters (primarily modulators) are much more important and baked directly into a proper DSP pipeline for radio transmission and reception

use super::pipeline_thread::PipelineThread;
//use super::dummy::{dummy_thread_function, DummyManager, DummyRunner};
use std::thread::{self, JoinHandle};
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crossbeam_queue::SegQueue;
use super::api::*;


pub type ConstructionQueue = Arc<SegQueue<PipelineThread>>;


#[derive(Clone)]
pub struct PipelineParameters {
    pub retries: usize,
    pub timeout: u64,
    pub max_infrastructure_errors: usize,
    pub max_compute_errors: usize,
    pub unchanged_state_time: u64,
    pub backpressure_val: usize
}
impl PipelineParameters {
    pub fn new(retries: usize, timeout: u64, backpressure_val: usize, max_infrastructure_errors: usize, max_compute_errors: usize, unchanged_state_time: u64) -> PipelineParameters {
        Self {
            retries,
            timeout,
            backpressure_val,
            max_compute_errors,
            max_infrastructure_errors,
            unchanged_state_time
        }
    }
}


pub struct ConstructingPipeline {
    nodes: ConstructionQueue,
    pub parameters: PipelineParameters
}
impl ConstructingPipeline {
    pub fn new(retries: usize, timeout: u64, backpressure_val: usize, max_infrastructure_errors: usize, max_compute_errors: usize, unchanged_state_time: u64) -> Self {
        let parameters = PipelineParameters::new(retries, timeout, backpressure_val, max_infrastructure_errors, max_compute_errors, unchanged_state_time);
        Self {
            nodes: Arc::new(SegQueue::new()),
            parameters
        }
    }
    pub fn get_nodes(&self) -> ConstructionQueue {
        self.nodes.clone()
    }
    pub fn finish_pipeline(mut self) -> ActivePipeline {
        let mut static_nodes = Vec::with_capacity(self.nodes.len());
        
        while self.nodes.len() > 0 {
            static_nodes.push(self.nodes.pop().unwrap());
        }
        
        ActivePipeline { nodes: static_nodes, parameters: self.parameters }
    }
}


pub struct ActivePipeline {
    nodes: Vec<PipelineThread>,
    parameters: PipelineParameters
}
impl ActivePipeline {
    pub fn start(&mut self) {
        log_message(format!("Starting active pipeline length: {}", self.nodes.len()), Level::Debug);
        //self.dummy_manager.start();
        for node in self.nodes.iter_mut() {
            node.start();
        }
    }
    
    pub fn stop(&mut self) {
        //self.dummy_manager.stop();
        log_message(format!("Stopping active pipeline length: {}", self.nodes.len()), Level::Debug);
        for node in self.nodes.iter_mut() {
            node.stop();
        }
    }
    
    pub fn kill(&mut self) {
        log_message(format!("Killing active pipeline length: {}", self.nodes.len()), Level::Debug);
        for mut node in self.nodes.iter_mut() {
            node.kill();
        }
    }
}