#[cfg(test)]
mod node_tests {
    use std::sync::mpsc;
    use crate::pipeline::pipeline::RadioPipeline;
    use crate::pipeline::pipeline_step::{joint_feedback_begin, PipelineNode, PipelineStep};
    use crate::pipeline::pipeline_traits::{Source, Sink};
    use crate::pipeline::pipeline_comms::ReceiveType;


    struct Dummy1 {
        receiver: mpsc::Receiver<u32>
    }
    impl PipelineStep<(), u32> for Dummy1 {
        fn run(&mut self, input: ReceiveType<()>) -> Result<u32, String> {
            let real_input = self.receiver.recv_timeout(std::time::Duration::from_millis(10000)).unwrap(); // note that this timeout is purposely set high
            Ok(real_input + 1) // why is timeout very large? because the first iteration of a feedback system needs to generate a dummy value on the feedback joint
            // which can take a bit of time for it to figure itself out. After the first input, youre safe to reduce this to much lower (assuming you have stupid/no error handling)
        }
    }
    impl Source for Dummy1 {}

    struct Dummy2{}
    impl PipelineStep<u32, u32> for Dummy2 {
        fn run(&mut self, input: ReceiveType<u32>) -> Result<u32, String> {
            match input {
                ReceiveType::Single(t) => Ok(t + 1),
                ReceiveType::Multi(t) => {
                    Ok(t.iter().sum())
                }
                _ => Err("Dummy".to_string())
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
    fn test_feedback_pipeline_assembly() { 
        // oftentimes it will be easier to contain a feedback loop directly inside of the pipeline step rather than constructing one within the 
        // framework of several steps as shown below. Feedback internal to a step offers finer grained control, and doesnt lose any performance
        // however if you really want supreme super separation of concerns because you're a good engineer feel free to use this for macro scale feedback loops
        // also if you dont want to keep manually reimplementing output feedback logic. Input feedback logic you should do yourself. But that isnt so hard. Is it?
        
        // can y[n-1] (on the pipeline scale). Mathematically this either means true y[n-1] in the case of a scalar type, or between
        // y[n - 1], y[n - k] for a k-sized vector type 
        let mut pipeline = RadioPipeline::new(3, 1000, 1);

        let input_pair = mpsc::sync_channel(1);
        let (output_sender, output_receiver) = mpsc::channel();
        
        let mut feedback_joint = joint_feedback_begin("Test Feedback Joint".to_string(), &mut pipeline);

        PipelineNode::start_pipeline(String::from("test_source"), Dummy1 {receiver: input_pair.1}, &mut pipeline)
            .attach(String::from("step 1"), Dummy2 {}, &mut pipeline)
            .branch_end("Test input branch end".to_string(), &mut feedback_joint);
        
        let mut lazy_node = feedback_joint.joint_add_lazy(&mut pipeline);
        
        let mut test_split = feedback_joint.joint_lock(Dummy2 {},  &mut pipeline)
            .split_begin("Test Split".to_string());
        
        test_split.split_add("Exit Branch".to_string(), &mut pipeline)
            .cap_pipeline("Exit".to_string(), Dummy3 {sender: output_sender}, &mut pipeline);
        
        lazy_node.joint_link_lazy("Feedback Node".to_string(), Dummy2 {}, test_split.split_add("Feedback Arm".to_string(), &mut pipeline), &mut pipeline);
        
        test_split.split_lock(Dummy2 {}, &mut pipeline);

        pipeline.start();

        input_pair.0.send(1);
        let result = output_receiver.recv().unwrap();
        dbg!(&result);
        assert_eq!(result, 4);
        
        input_pair.0.send(1);
        let result = output_receiver.recv().unwrap();
        dbg!(&result);
        assert_eq!(result, 9);
        
        pipeline.kill(); 
    }
}