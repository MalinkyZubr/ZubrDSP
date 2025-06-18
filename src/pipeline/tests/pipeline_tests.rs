#[cfg(test)]
mod node_tests {
    use std::sync::mpsc;
    use crate::pipeline::pipeline::RadioPipeline;
    use crate::pipeline::prototype::{PipelineNode, PipelineStep};
    use super::*;
    
    
    struct Dummy1{
        receiver: mpsc::Receiver<u8>
    }
    impl PipelineStep<(), u8> for Dummy1 {
        fn run(&mut self, input: ()) -> u8 {
            let input = self.receiver.recv().unwrap();
            input + 1
        }
    }

    struct Dummy2{}
    impl PipelineStep<u8, u8> for Dummy2 {
        fn run(&mut self, input: u8) -> u8 {
            input + 1
        }
    }

    struct Dummy3{}
    impl PipelineStep<u8, f32> for Dummy3 {
        fn run(&mut self, input: u8) -> f32 {
            return (input + 1) as f32;
        }
    }

    struct Dummy4{
        sender: mpsc::Sender<f32>,
    }
    impl PipelineStep<f32, ()> for Dummy4 {
        fn run(&mut self, input: f32) -> () {
            self.sender.send(input).unwrap();
        }
    }
    
    #[test]
    fn test_pipeline_assembly() {
        let mut pipeline = RadioPipeline::new();
        
        PipelineNode::start_pipeline()
    }
}
    