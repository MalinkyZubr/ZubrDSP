#[cfg(test)]
mod node_tests {
    use std::sync::mpsc;
    use std::thread::sleep;
    use crate::pipeline::pipeline::RadioPipeline;
    use crate::pipeline::pipeline_step::{PipelineNode, PipelineStep};
    use crate::pipeline::pipeline_traits::{Source, Sink};
    use crate::pipeline::pipeline_comms::{ReceiveType, ODFormat};
    use super::*;


    struct Dummy1 {
        receiver: mpsc::Receiver<u32>
    }
    impl PipelineStep<(), u32> for Dummy1 {
        fn run_SISO(&mut self, input: ()) -> Result<ODFormat<u32>, String> {
            let real_input = self.receiver.recv_timeout(std::time::Duration::from_millis(2000)).unwrap_or(0);
            Ok(ODFormat::Standard(real_input + 1))
        }
    }
    impl Source for Dummy1 {}

    struct Dummy2{}
    impl PipelineStep<u32, u32> for Dummy2 {
        fn run_SISO(&mut self, input: u32) -> Result<ODFormat<u32>, String> {
            Ok(ODFormat::Standard(input + 1))
        }
    }

    struct Dummy3{
        sender: mpsc::Sender<u32>,
    }
    impl PipelineStep<u32, ()> for Dummy3 {
        fn run_SISO(&mut self, input: u32) -> Result<ODFormat<()>, String> {
            self.sender.send(input).unwrap();
            Ok(ODFormat::Standard(()))
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
    