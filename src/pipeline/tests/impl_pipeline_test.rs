#[cfg(test)]
mod node_tests {
    use std::sync::mpsc;
    use std::thread::sleep;
    use num::Complex;
    use crate::dsp::fft::bit_reversal::FFTBitReversal;
    use crate::dsp::fft::bit_reversal_optimized::FFTBitReversalOptimized;
    use crate::pipeline::pipeline::RadioPipeline;
    use crate::pipeline::prototype::{PipelineNode, PipelineStep, Unit, Source, Sink};
    use super::*;


    struct Dummy1{
        counter: u32
    }
    impl PipelineStep<(), Vec<u32>> for Dummy1 {
        fn run(&mut self, input: Option<()>) -> Vec<u32> {
            if self.counter < 1000 {
                self.counter += 1;
                (0..2048).collect()
            }
            else {
                vec![]
            }
        }
    }
    impl Source for Dummy1 {}

    struct Dummy2{}
    impl PipelineStep<Vec<u32>, Vec<Complex<f32>>> for Dummy2 {
        fn run(&mut self, input: Option<Vec<u32>>) -> Vec<Complex<f32>> {
            input.unwrap().iter().map(|x| Complex::new(*x as f32, 0.0)).collect()
        }
    }

    struct Dummy5{
        sender: mpsc::SyncSender<Vec<u32>>,
    }
    impl PipelineStep<Vec<Complex<f32>>, ()> for Dummy5 {
        fn run(&mut self, input: Option<Vec<Complex<f32>>>) -> () {
            self.sender.send(input.unwrap().iter().map(|x| x.re as u32).collect()).unwrap();
        }
    }
    impl Sink for Dummy5 {}

    #[test]
    fn test_pipeline_fft() {
        let mut pipeline = RadioPipeline::new();

        let output_pair = mpsc::sync_channel(10);

        PipelineNode::start_pipeline(String::from("test_source"), Dummy1 {counter: 0}, &mut pipeline)
            .attach(String::from("step 1"), Dummy2 {}, &mut pipeline)
            .attach(String::from("step 2"), FFTBitReversalOptimized::new(2048, 8, false), &mut pipeline)
            .attach(String::from("step 2"), FFTBitReversalOptimized::new(2048, 8, true), &mut pipeline)
            .cap_pipeline(String::from("step 3"), Dummy5 { sender: output_pair.0 }, &mut pipeline);

        pipeline.start();

        for x in 0..1000 {
            let tester: Vec<u32> = (0..2048).collect();
            for (real, test) in output_pair.1.recv().unwrap().iter().zip(tester.iter()) {
                //dbg!(real, test);
                assert!((*real == 0 || *test == 0) || ((100.0 * (*real as f32 - *test as f32).abs() / *test as f32) < 10.0));
            }
        }

        pipeline.kill();
    }
}
