use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::pipeline::pipeline::RadioPipeline;
use super::pipeline_thread::PipelineThread;


pub trait Sharable = Send + Sync + Debug + 'static;
pub trait NotUnit {}
impl !NotUnit for () {}


pub trait PipelineStep<I: Send, O: Send> : Send {
    fn run(&mut self, input: I) -> O;
}

pub trait CallableNode<I: Sharable, O: Sharable> : Send {
    fn call(&mut self, step: &mut impl PipelineStep<I, O>);
} 

pub trait HasID {
    fn get_id(&self) -> String;
    fn set_id(&mut self, id: String);
}

pub struct PipelineNode<I: Sharable, O: Sharable> {
    pub input: Option<Receiver<I>>,
    pub output: Option<Sender<O>>,
    pub id: String
}

impl<I: Sharable, O: Sharable> HasID for PipelineNode<I, O> {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }
}

pub mod PipelineSource {
    use super::*;
    impl<O: Sharable + NotUnit> CallableNode<(), O> for PipelineNode<(), O> {
        fn call(&mut self, step: &mut impl PipelineStep<(), O>) {
            let output_data: O = step.run(());

            match self.output.as_mut().unwrap().send(output_data) {
                Ok(()) => {}
                Err(_msg) => {}
            }
        }
    }
    impl<O: Sharable + NotUnit> PipelineNode<(), O> {
        pub fn start_pipeline<F: Sharable + NotUnit>(start_id: String, source_step: impl PipelineStep<(), O> + 'static, pipeline: &mut RadioPipeline) -> PipelineNode<O, F> {
            let (sender, receiver) = channel::<O>();
            let start_node = PipelineNode { input: None, output: Some(sender), id: start_id };

            let mut successor: PipelineNode<O, F> = PipelineNode::new(String::from(""));
            successor.input = Some(receiver);

            let new_thread = PipelineThread::new(source_step, start_node);
            pipeline.nodes.push(new_thread);

            return successor;
        }
    }
}

pub mod PipelineSink {
    use super::*;

    impl<I: Sharable + NotUnit> CallableNode<I, ()> for PipelineNode<I, ()> {
        fn call(&mut self, step: &mut impl PipelineStep<I, ()>) {
            let input_data: I = self.input.as_mut().unwrap().recv().unwrap();
            step.run(input_data);
        }
    }

    impl<I: Sharable + NotUnit> PipelineNode<I, ()> {
        pub fn cap_pipeline(mut self, id: String, step: impl PipelineStep<I, ()> + 'static, pipeline: &mut RadioPipeline) {
            self.set_id(id);

            let new_thread = PipelineThread::new(step, self);
            pipeline.nodes.push(new_thread);
        }
    }
}


pub mod PipelineInner {
    use super::*;

    impl<I: Sharable + NotUnit, O: Sharable + NotUnit> CallableNode<I, O> for PipelineNode<I, O> {
        fn call(&mut self, step: &mut impl PipelineStep<I, O>) {
            let input_data: I = self.input.as_mut().unwrap().recv().unwrap();
            let output_data: O = step.run(input_data);

            match self.output.as_mut().unwrap().send(output_data) {
                Ok(()) => {}
                Err(_msg) => {}
            }
        }
    }


    impl<I: Sharable + NotUnit, O: Sharable + NotUnit> PipelineNode<I, O> {
        pub fn new(id: String) -> PipelineNode<I, O> {
            PipelineNode {
                input: None,
                output: None,
                id
            }
        }

        pub fn attach<F: Sharable + NotUnit>(mut self, id: String, step: impl PipelineStep<I, O> + 'static, pipeline: &mut RadioPipeline) -> PipelineNode<O, F> {
            let (sender, receiver) = channel::<O>();
            let mut successor: PipelineNode<O, F> = PipelineNode::new(String::from(""));

            self.set_id(id);

            self.output = Some(sender);
            successor.input = Some(receiver);

            let new_thread = PipelineThread::new(step, self);
            pipeline.nodes.push(new_thread);

            return successor;
        }
    }
}