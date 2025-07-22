use std::fmt::Debug;
use std::sync::mpsc;
use std::sync::mpsc::{RecvTimeoutError};
use crate::pipeline::pipeline::RadioPipeline;
use super::pipeline_thread::PipelineThread;
use super::pipeline_traits::{Sharable, Unit, HasID, Source, Sink};
use super::pipeline_comms::{WrappedReceiver, NodeReceiver, NodeSender, MultiReceiver, MultiSender, ReceiveType, SingleReceiver};


#[derive(Debug)]
pub enum PipelineStepResult {
    Success,
    SendError,
    RecvTimeoutError(RecvTimeoutError),
    ComputeError(String)
}

pub trait PipelineStep<I: Sharable, O: Sharable> : Send + 'static {
    fn run(&mut self, input: ReceiveType<I>) -> Result<O, String>;
}


#[derive(Debug, Copy, Clone)]
struct DummyStep {}
impl<T: Sharable> PipelineStep<T, T> for DummyStep {
    fn run(&mut self, input: ReceiveType<T>) -> Result<T, String> {
        match input {
            ReceiveType::Single(t) => Ok(t),
            ReceiveType::Multi(_) => Err(String::from("Received multi message from pipeline step")),
            ReceiveType::Dummy => Err(String::from("Dummy Value")),
        }
    }
}

pub trait CallableNode<I: Sharable, O: Sharable> : Send {
    fn call(&mut self, step: &mut impl PipelineStep<I, O>) -> PipelineStepResult;
}


pub struct PipelineNode<I: Sharable, O: Sharable> {
    pub input: NodeReceiver<I>,
    pub output: NodeSender<O>,
    pub id: String,
}

impl<I: Sharable, O: Sharable> HasID for PipelineNode<I, O> {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }
}


impl<I: Sharable, O: Sharable> CallableNode<I, O> for PipelineNode<I, O> {
    fn call(&mut self, step: &mut impl PipelineStep<I, O>) -> PipelineStepResult {
        let received_result = self.input.receive();
        //println!("{}======={:?}", &self.id, &received_result);
        match received_result {
            Err(err) => PipelineStepResult::RecvTimeoutError(err), // must have a way to handle if it is a dummy
            Ok(val) => {
                let output_data: Result<O, String> = step.run(val);
                self.compute_handler(output_data)
            }
        }
    }
}


impl<I: Sharable, O: Sharable> PipelineNode<I, O> {
    pub fn new() -> PipelineNode<I, O> {
        PipelineNode {
            input: NodeReceiver::Dummy,
            output: NodeSender::Dummy,
            id: String::from(""),
        }
    }

    fn compute_handler(&mut self, output_data: Result<O, String>) -> PipelineStepResult {
        match output_data {
            Err(err) => PipelineStepResult::ComputeError(err),
            Ok(extracted_data) => match self.output.send(extracted_data) {
                Err(_) => PipelineStepResult::SendError,
                Ok(_) => PipelineStepResult::Success
            }
        }
    }

    pub fn attach<F: Sharable>(mut self, id: String, step: impl PipelineStep<I, O> + 'static, pipeline: &mut RadioPipeline) -> PipelineNode<O, F> {
        // attach a step to the selected node (self) and create a thread
        // produce a successor node to continue the pipeline
        let (sender, receiver) = mpsc::sync_channel::<O>(pipeline.backpressure_val);
        let mut successor: PipelineNode<O, F> = PipelineNode::new();

        self.set_id(id);

        self.output = NodeSender::SO(sender);
        successor.input = NodeReceiver::SI(SingleReceiver::new(WrappedReceiver::new(receiver), pipeline.timeout, pipeline.retries));

        let new_thread = PipelineThread::new(step, self);
        pipeline.nodes.push(new_thread);

        return successor;
    }

    pub fn cap_pipeline(mut self, id: String, step: impl PipelineStep<I, O> + 'static + Sink, pipeline: &mut RadioPipeline)
    where O: Unit {
        // End a linear pipeline branch, allowing the step itself to handle output to other parts of the program
        self.set_id(id);

        self.output = NodeSender::Dummy;

        let new_thread = PipelineThread::new(step, self);
        pipeline.nodes.push(new_thread);
    }

    pub fn start_pipeline<F: Sharable>(start_id: String, source_step: impl PipelineStep<I, O> + 'static + Source, pipeline: &mut RadioPipeline) -> PipelineNode<O, F>
    where I: Unit {
        // start a pipeline, allowing the step itself to handle input from other parts of the program
        let (sender, receiver) = mpsc::sync_channel::<O>(pipeline.backpressure_val);

        let start_node: PipelineNode<I, O> = PipelineNode { 
            input: NodeReceiver::Dummy, 
            output: NodeSender::SO(sender), 
            id: start_id,
        };

        let mut successor: PipelineNode<O, F> = PipelineNode::new();
        successor.input = NodeReceiver::SI(SingleReceiver::new(WrappedReceiver::new(receiver), pipeline.timeout, pipeline.retries));

        let new_thread = PipelineThread::new(source_step, start_node);
        pipeline.nodes.push(new_thread);

        return successor;
    }
    
    pub fn split_begin(mut self, id: String) -> PipelineNode<I, O> {
        // take the node outputted by a previous step in the builder and declare it as multiple out
        // allows the node to have multiple outputs appended
        self.set_id(id);
        let sender: MultiSender<O> = MultiSender::new();
        self.output = NodeSender::MO(sender);

        self
    }

    pub fn split_add<F: Sharable>(&mut self, branch_name: String, pipeline: &mut RadioPipeline) -> PipelineNode<O, F> {
        // equivalent of start_pipeline for a subbranch of a flow diagram. generates an entry in the split for the branch
        // returns the head of the new branch which can be attached to like a normal linear pipeline
        match &mut self.output {
            NodeSender::MO(node_sender) => {
                let (split_sender, split_receiver) = mpsc::sync_channel::<O>(pipeline.backpressure_val);
                node_sender.add_sender(split_sender);
                
                let (successor_sender, successor_receiver) = mpsc::sync_channel::<O>(pipeline.backpressure_val);
                
                let dummy_receiver = NodeReceiver::SI(SingleReceiver::new(WrappedReceiver::new(split_receiver), pipeline.timeout,  pipeline.retries));
                let dummy_attacher_node: PipelineNode<O, O> = PipelineNode { input: dummy_receiver, output: NodeSender::SO(successor_sender), id: branch_name };
                
                let mut successor: PipelineNode<O, F> = PipelineNode::new();
                
                successor.input = NodeReceiver::SI(SingleReceiver::new(WrappedReceiver::new(successor_receiver), pipeline.timeout, pipeline.retries));

                let dummy_step = DummyStep {};
                let new_thread = PipelineThread::new(dummy_step, dummy_attacher_node);
                pipeline.nodes.push(new_thread);

                return successor;
            }
            _ => panic!("To add a split branch you must declare a node as a splitter with split_begin!")
        }
    }

    pub fn split_lock(self, step: impl PipelineStep<I, O> + 'static, pipeline: &mut RadioPipeline) {
        // submit the split to the thread pool, preventing any more branches from being added and making it computable
        let new_thread = PipelineThread::new(step, self);
        pipeline.nodes.push(new_thread);
    }
    
    pub fn branch_end(mut self, id: String, joint: &mut PipelineNode<I, O>) {
        self.set_id(id);
        
        match self.input {
            NodeReceiver::MI(_) => panic!("Cannot end branch with multiple in"),
            NodeReceiver::SI(receiver) => joint.joint_add(receiver.extract_receiver()),
            NodeReceiver::Dummy => panic!("Cannot end branch with Dummy"),
        }
    }
    
    pub fn joint_begin<JI: Sharable, JO: Sharable>(&mut self, id: String, pipeline: &mut RadioPipeline) -> PipelineNode<JI, JO> {
        // create a node marked as a join which can take multiple input receivers. used to join multiple sub branches together (eg adder or something)
        let mut joint_node: PipelineNode<JI, JO> = PipelineNode { input: NodeReceiver::Dummy, output: NodeSender::Dummy,  id: id };
        joint_node.input = NodeReceiver::MI(MultiReceiver::new(pipeline.timeout, pipeline.retries));
        
        joint_node
    }
    
    fn joint_add(&mut self, receiver: WrappedReceiver<I>) {
        // attach an input to a joint 
        match &mut self.input {
            NodeReceiver::MI(node_receiver) => { node_receiver.add_receiver(receiver) }
            _ => panic!("Cannot add a joint input to a node which was not declared as a joint with joint_begin")
        }
    }
    
    pub fn joint_lock<F: Sharable>(mut self, step: impl PipelineStep<I, O> + 'static, pipeline: &mut RadioPipeline) -> PipelineNode<O, F> {
        match &mut self.input {
            NodeReceiver::MI(_) => {
                let (sender, receiver) = mpsc::sync_channel::<O>(pipeline.backpressure_val);
                let mut successor: PipelineNode<O, F> = PipelineNode::new();

                self.output = NodeSender::SO(sender);
                successor.input = NodeReceiver::SI(SingleReceiver::new(WrappedReceiver::new(receiver), pipeline.timeout, pipeline.retries));

                let new_thread = PipelineThread::new(step, self);
                pipeline.nodes.push(new_thread);

                successor
            }
            _ => panic!("To joint lock a node it must be declared as a joint")
        }
    }
    
    pub fn joint_add_lazy<F: Sharable>(&mut self, pipeline: &RadioPipeline) -> PipelineNode<F, I> {
        // creates an empty placeholder node for a joint that can be made concrete later to facilitate feedback architecture
        let (sender, receiver) = mpsc::sync_channel::<I>(pipeline.backpressure_val);
        match &mut self.input {
            NodeReceiver::MI(node_receiver) => node_receiver.add_receiver(WrappedReceiver::new(receiver).set_startup_flag()),
            _ => panic!("Cannot add lazy feedback node to a node which was not declared as a joint with joint_begin")
        };
        
        let mut lazy_node = PipelineNode::new();
        lazy_node.output = NodeSender::SO(sender);
        
        lazy_node
    }
    
    // pub fn joint_link_lazy<F: Sharable>(mut self, id: String, step: impl PipelineStep<I, O>, mut lazy_node: PipelineNode<O, F>, pipeline: &mut RadioPipeline) -> PipelineNode<O, F> {
    //     // takes the final node of a branch and attaches it to a lazy node's input. You must still assign the lazy node input with joint_lazy_finalize
    //     let (sender, receiver) = mpsc::sync_channel::<O>(pipeline.backpressure_val);
    // 
    //     self.set_id(id);
    // 
    //     self.output = NodeSender::SO(sender);
    //     lazy_node.input = NodeReceiver::SI(SingleReceiver::new(WrappedReceiver::new(receiver), pipeline.timeout, pipeline.retries));
    // 
    //     let new_thread = PipelineThread::new(step, self);
    //     pipeline.nodes.push(new_thread);
    //     
    //     lazy_node
    // }
    // 
    // pub fn joint_lazy_finalize(mut self, id: String, step: impl PipelineStep<I, O>, pipeline: &mut RadioPipeline) {
    //     // takes the computation step for a lazy node to close the feedback loop and submit it to the thread pool
    //     self.id = id;
    //     let new_thread = PipelineThread::new(step, self);
    //     pipeline.nodes.push(new_thread);
    // }
    
    pub fn joint_link_lazy(mut self, id: String, step: impl PipelineStep<I, O>, source_node: PipelineNode<I, O>, pipeline: &mut RadioPipeline) {
        // takes the final node of a branch and attaches it to a lazy node's input. You must still assign the lazy node input with joint_lazy_finalize
        self.set_id(id);
        
        match source_node.input {
            NodeReceiver::SI(receiver) => {
                self.input = NodeReceiver::SI(receiver);
                let new_thread = PipelineThread::new(step, self);
                pipeline.nodes.push(new_thread);
            }
            _ => panic!("Feedback joint cannot handle multiple input previous node"),
        }
    }
}

pub fn joint_feedback_begin<I: Sharable, O: Sharable>(id: String, pipeline: &mut RadioPipeline) -> PipelineNode<I, O> {
    // Since there is no convenient origin point for a joint used in feedback in the pattern, a standalone function is needed to support type inference
    let mut joint_node: PipelineNode<I, O> = PipelineNode { input: NodeReceiver::Dummy, output: NodeSender::Dummy,  id: id };
    joint_node.input = NodeReceiver::MI(MultiReceiver::new(pipeline.timeout, pipeline.retries));

    joint_node
}
