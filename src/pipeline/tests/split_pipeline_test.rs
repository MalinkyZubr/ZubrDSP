#[cfg(test)]
mod node_tests {
    use std::sync::mpsc;
    use crate::pipeline::pipeline::RadioPipeline;
    use crate::pipeline::pipeline_step::{PipelineNode, PipelineStep, JointBuilder, joint_begin};
    use crate::pipeline::pipeline_traits::{Source, Sink};
    use crate::pipeline::pipeline_comms::{ReceiveType, ODFormat};


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
        fn run_MISO(&mut self, input: Vec<u32>) -> Result<ODFormat<u32>, String> {
            Ok(ODFormat::Standard(input.iter().sum()))
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
    fn test_branch_pipeline_assembly() {
        let mut pipeline = RadioPipeline::new(3, 1000, 1);

        let input_pair = mpsc::sync_channel(1);
        let (output_sender, output_receiver) = mpsc::channel();
        
        let mut split = PipelineNode::start_pipeline(String::from("test_source"), Dummy1 {receiver: input_pair.1}, &mut pipeline)
            .attach(String::from("step 1"), Dummy2 {}, &mut pipeline).split_begin("test_split".to_string());

        let mut test_joint = joint_begin("Test Joint".to_string(), &mut pipeline);
        
        split.split_add("Test Branch 1".to_string(), &mut pipeline)
            .attach("branch_1 node 1".to_string(), Dummy2 {},  &mut pipeline)
            .attach("branch_1 node 2".to_string(), Dummy2 {},  &mut pipeline)
            .branch_end("branch_1 node 3 end".to_string(), &mut test_joint);
        
        split.split_add("Test Branch 2".to_string(), &mut pipeline)
            .attach("branch_2 node 1".to_string(), Dummy2 {},  &mut pipeline)
            .branch_end("branch_2 node 2 end".to_string(), &mut test_joint);
        
        split.split_lock(Dummy2 {}, &mut pipeline);
        
        test_joint.joint_lock(Dummy2 {}, &mut pipeline)
            .attach("Post Joint Test Node".to_string(), Dummy2 {},  &mut pipeline)
            .cap_pipeline("Test Cap".to_string(), Dummy3 { sender: output_sender.clone() }, &mut pipeline);
        
        pipeline.start();

        input_pair.0.send(1).unwrap();
        let result = output_receiver.recv().unwrap();
        dbg!(&result);
        assert_eq!(result, 12);

        pipeline.kill();
    }
}