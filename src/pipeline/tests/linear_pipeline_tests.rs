#[cfg(test)]
mod node_tests {
    use std::sync::mpsc;
    use std::thread::sleep;
    use crate::pipeline::pipeline::RadioPipeline;
    use crate::pipeline::pipeline_step::{PipelineNode, PipelineStep};
    use crate::pipeline::pipeline_traits::{Source, Sink};
    use crate::pipeline::pipeline_comms::ReceiveType;
    use super::*;
    

    struct Dummy1 {
        receiver: mpsc::Receiver<u32>
    }
    impl PipelineStep<(), u32> for Dummy1{
        fn run(&mut self, input: ReceiveType<()>) -> Result<u32, String> {
            let real_input = self.receiver.recv_timeout(std::time::Duration::from_millis(100)).unwrap_or(0);
            Ok(real_input + 1)
        }
    }
    impl Source for Dummy1 {}

    struct Dummy2{}
    impl PipelineStep<u32, u32> for Dummy2 {
        fn run(&mut self, input: ReceiveType<u32>) -> Result<u32, String> {
            match input {
                ReceiveType::Single(t) => Ok(t + 1),
                _ => Err(String::from("Received multi message from pipeline step"))
            }
        }
    }

    struct Dummy3{
        sender: mpsc::Sender<u32>,
    }
    impl PipelineStep<u32, ()> for Dummy3 {
        fn run(&mut self, input: ReceiveType<u32>) -> Result<(), String> {
            match input {
                ReceiveType::Single(val) => {
                    self.sender.send(val).unwrap();
                    Ok(())
                }   
                _ => Err(String::from("Received multi message from pipeline step"))
            }
        }
    }
    impl Sink for Dummy3 {}
    
    #[test]
    fn test_pipeline_assembly() {
        let mut pipeline = RadioPipeline::new(3, 1000, 1);

        let input_pair = mpsc::sync_channel(1);
        let (output_sender, output_receiver) = mpsc::channel();
        
        PipelineNode::start_pipeline(String::from("test_source"), Dummy1 {receiver: input_pair.1}, &mut pipeline)
            .attach(String::from("step 1"), Dummy2 {}, &mut pipeline)
            .cap_pipeline(String::from("step 2"), Dummy3 {sender: output_sender}, &mut pipeline);

        pipeline.start();

        input_pair.0.send(1).unwrap();
        let result = output_receiver.recv().unwrap();
        dbg!(&result);
        assert_eq!(result, 3);

        input_pair.0.send(2).unwrap();
        let result = output_receiver.recv().unwrap();
        dbg!(&result);
        assert_eq!(result, 4);

        pipeline.kill();
    }
}
    