#[cfg(test)]
mod node_tests {
    use std::sync::mpsc;
    use std::thread::sleep;
    use crate::pipeline::pipeline::RadioPipeline;
    use crate::pipeline::pipeline_step::{PipelineNode, PipelineStep};
    use crate::pipeline::pipeline_traits::{Source, Sink};
    use super::*;
    
    
    struct Dummy1{
        receiver: mpsc::Receiver<u32>
    }
    impl PipelineStep<(), u32> for Dummy1 {
        fn run(&mut self, input: ()) -> u32 {
            let input = self.receiver.recv_timeout(std::time::Duration::from_millis(100)).unwrap_or(0);
            input + 1
        }
    }
    impl Source for Dummy1 {}

    struct Dummy2{}
    impl PipelineStep<u32, u32> for Dummy2 {
        fn run(&mut self, input: u32) -> u32 {
            input + 1
        }
    }

    struct Dummy3{}
    impl PipelineStep<u32, f32> for Dummy3 {
        fn run(&mut self, input: u32) -> f32 {
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
    impl Sink for Dummy4 {}
    
    #[test]
    fn test_pipeline_assembly() {
        let mut pipeline = RadioPipeline::new();

        let input_pair = mpsc::sync_channel(1);
        let output_pair = mpsc::channel();
        
        PipelineNode::start_pipeline(String::from("test_source"), Dummy1 {receiver: input_pair.1}, &mut pipeline)
            .attach(String::from("step 1"), Dummy2 {}, &mut pipeline)
            .attach(String::from("step 2"), Dummy3 {}, &mut pipeline)
            .cap_pipeline(String::from("step 3"), Dummy4 { sender: output_pair.0 }, &mut pipeline);

        pipeline.start();

        input_pair.0.send(1).unwrap();
        let result = output_pair.1.recv().unwrap();
        dbg!(&result);
        assert_eq!(result, 4.0);

        pipeline.kill();
    }

    #[test]
    fn test_pipeline_stoppage() {
        let mut pipeline = RadioPipeline::new();

        let input_pair = mpsc::sync_channel(5);
        let output_pair = mpsc::channel();

        PipelineNode::start_pipeline(String::from("test_source"), Dummy1 {receiver: input_pair.1}, &mut pipeline)
            .attach(String::from("step 1"), Dummy2 {}, &mut pipeline)
            .attach(String::from("step 2"), Dummy3 {}, &mut pipeline)
            .cap_pipeline(String::from("step 3"), Dummy4 { sender: output_pair.0 }, &mut pipeline);

        pipeline.start();

        for input in 1..200000 {
            input_pair.0.send(input).unwrap();
        }
        
        pipeline.stop();
        sleep(std::time::Duration::from_millis(2000));
        pipeline.start();
        
        for input in 1..200000 {
            assert_eq!(output_pair.1.recv().unwrap(), input as f32 + 3.0);
        }

        pipeline.kill();
    }
}
    