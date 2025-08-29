use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize};
use std::sync::mpsc;
use std::sync::mpsc::{RecvTimeoutError};
use std::sync::Arc;
use futures::future::Lazy;
use log::Level;
use crossbeam_queue::ArrayQueue;
use crate::pipeline::pipeline::{ConstructingPipeline, ConstructionQueue, PipelineParameters};
use super::pipeline_thread::PipelineThread;
use super::pipeline_traits::{Sharable, Unit, HasID, Source, Sink};
use super::pipeline_comms::{WrappedReceiver, NodeReceiver, NodeSender, MultichannelReceiver, MultichannelSender, ReceiveType, SingleReceiver, ODFormat, SingleSender, Reassembler, Multiplexer, Demultiplexer};
use super::api::*;


#[derive(Debug, PartialEq, Clone)]
pub enum PipelineStepResult {
    Success,
    SendError,
    RecvTimeoutError(RecvTimeoutError),
    ComputeError(String),
    Carryover
}

// how can I make multiple input and output types more convenient?
/*
1. every pipeline step has a separate trait method for each input type, with separate signature. By defualt it will return an error saying its unimplemented
2. user can return whatever data scheme they want from each separate handler for the node to do with what it pleases
3. at the beginning of runtime, depending on the receiver type assigned to the node, a different handler (node method) is chosen to receive, so no additional match is needed
 */
pub trait PipelineStep<I: Sharable, O: Sharable> : Send + 'static {
    // There is a single input, and a single output
    fn run_SISO(&mut self, input: I) -> Result<ODFormat<O>, String> { panic!("Single in Single Out Not Implemented") }
    // You are receiving a vector of values reassembled from a series branch, and are outputting to a single output
    fn run_REASO(&mut self, input: Vec<I>) -> Result<ODFormat<O>, String> { panic!("Series In Single Out Not Implemented") }
    // you are receiving a vector of values, each one representing the output of a distinct branch. Outputting to a single output
    fn run_MISO(&mut self, input: Vec<I>) -> Result<ODFormat<O>, String> { panic!("Multiple In Single Out Not Implemented") }
    // you are receiving a single value and outputting to multiple distinct pipeline branches
    fn run_SIMO(&mut self, input: I) -> Result<ODFormat<O>, String> { panic!("Single In Multiple Out Not Implemented") }
    // you are receiving a vector of values, representing a series receive, and outputing to multiple outputs
    fn run_REAMO(&mut self, input: Vec<I>) -> Result<ODFormat<O>, String> { panic!("Series In Multiple Out Not Implemented") }
    // receiving a vector of outputs from distinct pipeline branches and outputting to multiple distinct pipeline branches
    fn run_MIMO(&mut self, input: Vec<I>) -> Result<ODFormat<O>, String> { panic!("Multiple In Multiple Out Not Implemented") }
    // intended for source nodes. When the source receiver is 'Dummy'. This is a PRODUCER startpoint. Generates its own input
    fn run_DISO(&mut self) -> Result<ODFormat<O>, String> {
        panic!("Dummy in Not Implemented")
    }
    // intended for sink nodes. When the sink sender is 'Dummy'. This is a CONSUMER endpoint. Must put the output wherever it needs to go on its own
    fn run_SIDO(&mut self, input: I) -> Result<ODFormat<O>, String> { panic!("Dummy out Not Implemented") }
    // optional method to be run whenever a pause signal is received
    fn pause_behavior(&mut self) { () }
    // optional method to be run whenever a start signal is received
    fn start_behavior(&mut self) { () }
    // optional method to be run whenevr a kill signal is received
    fn kill_behavior(&mut self) { () }
}


#[derive(Debug, Copy, Clone)]
struct DummyStep {}
impl<T: Sharable> PipelineStep<T, T> for DummyStep {
    fn run_SISO(&mut self, input: T) -> Result<ODFormat<T>, String> {
        Ok(ODFormat::Standard(input))
    }
}


pub struct PipelineNode<I: Sharable, O: Sharable> {
    pub input: NodeReceiver<I>,
    pub output: NodeSender<O>,
    pub id: String,
    tap: Option<Arc<ArrayQueue<ODFormat<O>>>>
}

impl<I: Sharable, O: Sharable> HasID for PipelineNode<I, O> {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn set_id(&mut self, id: &str) {
        self.id = id.to_string();
    }
}


impl<I: Sharable, O: Sharable> PipelineNode<I, O> {
    pub fn new() -> PipelineNode<I, O> {
        PipelineNode {
            input: NodeReceiver::Dummy,
            output: NodeSender::Dummy,
            id: "".to_string(),
            tap: None
        }
    }

    pub fn call(&mut self, step: &mut impl PipelineStep<I, O>) -> PipelineStepResult {
        let received_result = self.input.receive();
        match received_result {
            Err(err) => {
                let result = PipelineStepResult::RecvTimeoutError(err);
                result
            }, // must have a way to handle if it is a dummy
            Ok(val) => {
                let result = self.route_computation(val, step);
                result
            }
        }
    }

    fn compute_handler(&mut self, output_data: Result<ODFormat<O>, String>) -> PipelineStepResult {
        match output_data {
            Err(err) => PipelineStepResult::ComputeError(err),
            Ok(extracted_data) => {
                let extracted_data = self.push_to_tap(extracted_data);
                match self.output.send(extracted_data) {
                    Err(_) => PipelineStepResult::SendError,
                    Ok(_) => {
                        PipelineStepResult::Success
                    }
                }
            }
        }
    }
    fn push_to_tap(&mut self, output_data: ODFormat<O>) -> ODFormat<O> {
        match &mut self.tap {
            None => output_data,
            Some(tap) => {
                let cloned_output = output_data.clone();
                tap.push(output_data);
                cloned_output
            }
        }
    }
    
    fn route_computation(&mut self, input_data: ReceiveType<I>, step: &mut impl PipelineStep<I, O>) -> PipelineStepResult {
        //log_message(format!("CRITICAL: NodeID: {}, Received message: {}", &self.id, &input_data), Level::Debug);
        self.compute_handler(match (input_data, &self.output) {
            (ReceiveType::Single(t), NodeSender::SO(_) | NodeSender::MUO(_)) => step.run_SISO(t),
            (ReceiveType::Single(t), NodeSender::MO(_)) => step.run_SIMO(t),
            (ReceiveType::Multichannel(t), NodeSender::SO(_) | NodeSender::MUO(_)) => step.run_MISO(t),
            (ReceiveType::Multichannel(t), NodeSender::MO(_)) => step.run_MIMO(t),
            (ReceiveType::Reassembled(t), NodeSender::SO(_) | NodeSender::MUO(_)) => step.run_REASO(t),
            (ReceiveType::Reassembled(t), NodeSender::MO(_)) => step.run_REAMO(t),
            (ReceiveType::Dummy, NodeSender::SO(_)) => step.run_DISO(),
            (ReceiveType::Single(t), NodeSender::Dummy) => step.run_SIDO(t),
            (_, _) => Err(String::from("Received bad message from pipeline step")),
        })
    }
}


pub struct NodeBuilder<I: Sharable, O: Sharable> {
    node: PipelineNode<I, O>,
    construction_queue: ConstructionQueue,
    parameters: PipelineParameters,
    state: (Arc<AtomicU8>, mpsc::Sender<ThreadStateSpace>),
}
impl<I: Sharable, O: Sharable> NodeBuilder<I, O> {
    pub fn attach<F: Sharable>(mut self, id: &'static str, step: impl PipelineStep<I, O> + 'static) -> NodeBuilder<O, F> {
        // attach a step to the selected node (self) and create a thread
        // produce a successor node to continue the pipeline
        let (sender, receiver) = mpsc::sync_channel::<O>(self.parameters.backpressure_val);
        let mut successor: PipelineNode<O, F> = PipelineNode::new();

        self.node.set_id(id);

        self.node.output = NodeSender::SO(SingleSender::new(sender));
        successor.input = NodeReceiver::SI(SingleReceiver::new(WrappedReceiver::new(receiver), self.parameters.timeout, self.parameters.retries));

        let new_thread = PipelineThread::new(step, self.node, self.parameters.clone(), self.state.clone());
        self.construction_queue.push(new_thread);

        NodeBuilder { node: successor, construction_queue: self.construction_queue, parameters: self.parameters, state: self.state}
    }

    pub fn cap_pipeline(mut self, id: &'static str, step: impl PipelineStep<I, O> + 'static + Sink)
    where O: Unit {
        // End a linear pipeline branch, allowing the step itself to handle output to other parts of the program
        self.node.set_id(id);

        self.node.output = NodeSender::Dummy;

        let new_thread = PipelineThread::new(step, self.node, self.parameters.clone(), self.state);
        self.construction_queue.push(new_thread);
    }

    pub fn start_pipeline<F: Sharable>(start_id: &str, source_step: impl PipelineStep<I, O> + 'static + Source, pipeline: &ConstructingPipeline) -> NodeBuilder<O, F>
    where I: Unit {
        let parameters = pipeline.get_cloned_parameters();
        // start a pipeline, allowing the step itself to handle input from other parts of the program
        let (sender, receiver) = mpsc::sync_channel::<O>(parameters.backpressure_val);

        let start_node: PipelineNode<I, O> = PipelineNode {
            input: NodeReceiver::Dummy,
            output: NodeSender::SO(SingleSender::new(sender)),
            id: start_id.to_string(),
            tap: None,
        };

        let mut successor: PipelineNode<O, F> = PipelineNode::new();
        successor.input = NodeReceiver::SI(SingleReceiver::new(WrappedReceiver::new(receiver), parameters.timeout, parameters.retries));

        let new_thread = PipelineThread::new(source_step, start_node, parameters.clone(), pipeline.get_state_communicators());
        pipeline.get_nodes().push(new_thread);

        NodeBuilder { node: successor, parameters, construction_queue: pipeline.get_nodes(), state: pipeline.get_state_communicators() }
    }

    pub fn split_begin(mut self, id: &'static str) -> SplitBuilder<I, O> {
        // take the node outputted by a previous step in the builder and declare it as multiple out
        // allows the node to have multiple outputs appended
        self.node.set_id(id);
        let sender: MultichannelSender<O> = MultichannelSender::new();
        self.node.output = NodeSender::MO(sender);

        SplitBuilder { node: self.node, parameters: self.parameters, construction_queue: self.construction_queue, state: self.state.clone() }
    }

    pub fn mutltiplexer_begin(mut self, id: &'static str, channel_selector: Arc<AtomicUsize>) -> MultiplexerBuilder<I, O> {
        self.node.set_id(id);
        let sender: Multiplexer<O> = Multiplexer::new(channel_selector);
        self.node.output = NodeSender::MUO(sender);

        MultiplexerBuilder { node: self.node, parameters: self.parameters, construction_queue: self.construction_queue, state: self.state }
    }

    pub fn branch_end(mut self, joint_builder: &mut JointBuilder<I, O>) {
        match self.node.input {
            NodeReceiver::SI(receiver) => joint_builder.joint_add(receiver.extract_receiver()),
            NodeReceiver::Dummy => panic!("Cannot end branch with Dummy"),
            _ => panic!("Must end branch with single. This should be automatic behavior")
        }
    }
    
    pub fn multiplex_branch_end(mut self, demultiplexer_builder: &mut DemultiplexerBuilder<I, O>) {
        match self.node.input {
            NodeReceiver::SI(receiver) => demultiplexer_builder.demultiplexer_add(receiver.extract_receiver()),
            NodeReceiver::Dummy => panic!("Cannot end multiplexed branch with Dummy"),
            _ => panic!("Must end branch with single. This should be automatic behavior")
        }
    }

    pub fn add_reassembler(mut self, reassemble_quantity: usize) -> Self {
        match self.node.input {
            NodeReceiver::SI(receiver) => {
                self.node.input = NodeReceiver::REA(Reassembler::new(receiver.extract_receiver(), reassemble_quantity, self.parameters.timeout, self.parameters.retries))
            }
            _ => panic!("Cannot set reassembler on this node, must be a single receiver node"),
        }

        self
    }
}


pub fn demultiplexer_begin<JI: Sharable, JO: Sharable>(id: &str, channel_selector: Arc<AtomicUsize>, pipeline: &ConstructingPipeline) -> DemultiplexerBuilder<JI, JO> {
    // create a node marked as a join which can take multiple input receivers. used to join multiple sub branches together (eg adder or something)
    let mut demultiplexer_node: PipelineNode<JI, JO> = PipelineNode { input: NodeReceiver::Dummy, output: NodeSender::Dummy, id: id.to_string(), tap: None };
    let parameters = pipeline.get_cloned_parameters();
    demultiplexer_node.input = NodeReceiver::DMI(Demultiplexer::new(channel_selector, parameters.timeout, parameters.retries));

    DemultiplexerBuilder { node: demultiplexer_node, parameters, construction_queue: pipeline.get_nodes(), state: pipeline.get_state_communicators() }
}

pub fn joint_begin<JI: Sharable, JO: Sharable>(id: &str, pipeline: &ConstructingPipeline) -> JointBuilder<JI, JO> {
    // create a node marked as a joint which can take multiple input receivers. used to join multiple sub branches together (eg adder or something)
    let mut joint_node: PipelineNode<JI, JO> = PipelineNode { input: NodeReceiver::Dummy, output: NodeSender::Dummy, id: id.to_string(), tap: None };
    let parameters = pipeline.get_cloned_parameters();
    joint_node.input = NodeReceiver::MI(MultichannelReceiver::new(parameters.timeout, parameters.retries));

    JointBuilder { node: joint_node, parameters, construction_queue: pipeline.get_nodes(), state: pipeline.get_state_communicators() }
}

pub fn joint_feedback_begin<I: Sharable, O: Sharable>(id: &str, pipeline: &ConstructingPipeline) -> JointBuilder<I, O> {
    // Since there is no convenient origin point for a joint used in feedback in the pattern, a standalone function is needed to support type inference
    let mut joint_node: PipelineNode<I, O> = PipelineNode { input: NodeReceiver::Dummy, output: NodeSender::Dummy, id: id.to_string(), tap: None };
    let parameters = pipeline.get_cloned_parameters();
    joint_node.input = NodeReceiver::MI(MultichannelReceiver::new(parameters.timeout, parameters.retries));

    JointBuilder { node: joint_node, parameters, construction_queue: pipeline.get_nodes(), state: pipeline.get_state_communicators() }
}


pub struct SplitBuilder<I: Sharable, O: Sharable> {
    node: PipelineNode<I, O>,
    construction_queue: ConstructionQueue,
    parameters: PipelineParameters,
    state: (Arc<AtomicU8>, mpsc::Sender<ThreadStateSpace>),
}
impl<I: Sharable, O: Sharable> SplitBuilder<I, O> {
    pub fn split_add<F: Sharable>(&mut self) -> NodeBuilder<O, F> {
        // equivalent of start_pipeline for a subbranch of a flow diagram. generates an entry in the split for the branch
        // returns the head of the new branch which can be attached to like a normal linear pipeline
        match &mut self.node.output {
            NodeSender::MO(node_sender) => {
                let (split_sender, split_receiver) = mpsc::sync_channel::<O>(self.parameters.backpressure_val);
                node_sender.add_sender(split_sender);

                let mut successor: PipelineNode<O, F> = PipelineNode::new();

                successor.input = NodeReceiver::SI(SingleReceiver::new(WrappedReceiver::new(split_receiver), self.parameters.timeout, self.parameters.retries));

                NodeBuilder { node: successor, parameters: self.parameters.clone(), construction_queue: self.construction_queue.clone(), state: self.state.clone() }
            }
            _ => panic!("To add a split branch you must declare a node as a splitter with split_begin!")
        }
    }

    pub fn split_lock(self, step: impl PipelineStep<I, O> + 'static) {
        // submit the split to the thread pool, preventing any more branches from being added and making it computable
        let new_thread = PipelineThread::new(step, self.node, self.parameters.clone(), self.state.clone());
        self.construction_queue.push(new_thread);
    }
}


pub struct LazyJointInputBuilder<I: Sharable, O: Sharable> {
    node:  PipelineNode<I, O>,
    construction_queue: ConstructionQueue,
    parameters: PipelineParameters,
    state: (Arc<AtomicU8>, mpsc::Sender<ThreadStateSpace>),
}
impl<I: Sharable, O: Sharable> LazyJointInputBuilder<I, O> {
    pub fn joint_link_lazy(mut self, id: &'static str, step: impl PipelineStep<I, O>, source_node: NodeBuilder<I, O>) {
        // takes the final node of a branch and attaches it to a lazy node's input. You must still assign the lazy node input with joint_lazy_finalize
        self.node.set_id(id);

        match source_node.node.input {
            NodeReceiver::SI(receiver) => {
                self.node.input = NodeReceiver::SI(receiver);
                let new_thread = PipelineThread::new(step, self.node, self.parameters.clone(), self.state.clone());
                self.construction_queue.push(new_thread);
            }
            _ => panic!("Feedback joint cannot handle multiple input previous node"),
        }
    }
}


pub struct JointBuilder<I: Sharable, O: Sharable> {
    node: PipelineNode<I, O>,
    construction_queue: ConstructionQueue,
    parameters: PipelineParameters,
    state: (Arc<AtomicU8>, mpsc::Sender<ThreadStateSpace>),
}
impl<I: Sharable, O: Sharable> JointBuilder<I, O> {
    fn joint_add(&mut self, receiver: WrappedReceiver<I>) {
        // attach an input to a joint
        match &mut self.node.input {
            NodeReceiver::MI(node_receiver) => { node_receiver.add_receiver(receiver) }
            _ => panic!("Cannot add a joint input to a node which was not declared as a joint with joint_begin")
        }
    }

    pub fn joint_lock<F: Sharable>(mut self, step: impl PipelineStep<I, O> + 'static) -> NodeBuilder<O, F> {
        match &mut self.node.input {
            NodeReceiver::MI(_) => {
                let (sender, receiver) = mpsc::sync_channel::<O>(self.parameters.backpressure_val);
                let mut successor: PipelineNode<O, F> = PipelineNode::new();

                self.node.output = NodeSender::SO(SingleSender::new(sender));
                successor.input = NodeReceiver::SI(SingleReceiver::new(WrappedReceiver::new(receiver), self.parameters.timeout, self.parameters.retries));

                let new_thread = PipelineThread::new(step, self.node, self.parameters.clone(), self.state.clone());
                self.construction_queue.push(new_thread);

                NodeBuilder { node: successor, parameters: self.parameters.clone(), construction_queue: self.construction_queue.clone(), state: self.state.clone() }
            }
            _ => panic!("To joint lock a node it must be declared as a joint")
        }
    }

    pub fn joint_add_lazy<F: Sharable>(&mut self) -> LazyJointInputBuilder<F, I> {
        // creates an empty placeholder node for a joint that can be made concrete later to facilitate feedback architecture
        let (sender, receiver) = mpsc::sync_channel::<I>(self.parameters.backpressure_val);
        match &mut self.node.input {
            NodeReceiver::MI(node_receiver) => node_receiver.add_receiver(WrappedReceiver::new(receiver).set_startup_flag()),
            _ => panic!("Cannot add lazy feedback node to a node which was not declared as a joint with joint_begin")
        };

        let mut lazy_node = PipelineNode::new();
        lazy_node.output = NodeSender::SO(SingleSender::new(sender));

        LazyJointInputBuilder { node: lazy_node, parameters: self.parameters.clone(), construction_queue: self.construction_queue.clone(), state: self.state.clone() }
    }
}

pub struct MultiplexerBuilder<I: Sharable, O: Sharable> {
    node: PipelineNode<I, O>,
    construction_queue: ConstructionQueue,
    parameters: PipelineParameters,
    state: (Arc<AtomicU8>, mpsc::Sender<ThreadStateSpace>),
}
impl<I: Sharable, O: Sharable> MultiplexerBuilder<I, O> {
    pub fn multiplexer_add<F: Sharable>(&mut self) -> NodeBuilder<O, F> {
        // equivalent of start_pipeline for a subbranch of a flow diagram. generates an entry in the split for the branch
        // returns the head of the new branch which can be attached to like a normal linear pipeline
        match &mut self.node.output {
            NodeSender::MUO(node_sender) => {
                let (multiplexer_sender, multiplexer_receiver) = mpsc::sync_channel::<O>(self.parameters.backpressure_val);
                node_sender.add_sender(multiplexer_sender);

                let mut successor: PipelineNode<O, F> = PipelineNode::new();

                successor.input = NodeReceiver::SI(SingleReceiver::new(WrappedReceiver::new(multiplexer_receiver), self.parameters.timeout, self.parameters.retries));
                NodeBuilder { node: successor, parameters: self.parameters.clone(), construction_queue: self.construction_queue.clone(), state: self.state.clone() }
            }
            _ => panic!("To add a multiplexer branch you must declare it as a multiplexer with multiplexer_start")
        }
    }
    pub fn multiplexer_lock(self, step: impl PipelineStep<I, O> + 'static) {
        // submit the split to the thread pool, preventing any more branches from being added and making it computable
        let new_thread = PipelineThread::new(step, self.node, self.parameters.clone(), self.state.clone());
        self.construction_queue.push(new_thread);
    }
}


pub struct DemultiplexerBuilder<I: Sharable, O: Sharable> {
    node: PipelineNode<I, O>,
    construction_queue: ConstructionQueue,
    parameters: PipelineParameters,
    state: (Arc<AtomicU8>, mpsc::Sender<ThreadStateSpace>),
}
impl<I: Sharable, O: Sharable> DemultiplexerBuilder<I, O> {
    fn demultiplexer_add(&mut self, receiver: WrappedReceiver<I>) {
        // attach an input to a joint
        match &mut self.node.input {
            NodeReceiver::DMI(node_receiver) => { node_receiver.add_receiver(receiver) }
            _ => panic!("Cannot add a demultiplexer input to a node which was not declared as a demultiplexer with demultiplexer_begin")
        }
    }

    pub fn demultiplexer_lock<F: Sharable>(mut self, step: impl PipelineStep<I, O> + 'static) -> NodeBuilder<O, F> {
        match &mut self.node.input {
            NodeReceiver::DMI(_) => {
                let (sender, receiver) = mpsc::sync_channel::<O>(self.parameters.backpressure_val);
                let mut successor: PipelineNode<O, F> = PipelineNode::new();

                self.node.output = NodeSender::SO(SingleSender::new(sender));
                successor.input = NodeReceiver::SI(SingleReceiver::new(WrappedReceiver::new(receiver), self.parameters.timeout, self.parameters.retries));

                let new_thread = PipelineThread::new(step, self.node, self.parameters.clone(), self.state.clone());
                self.construction_queue.push(new_thread);

                NodeBuilder { node: successor, parameters: self.parameters.clone(), construction_queue: self.construction_queue.clone(), state: self.state.clone() }
            }
            _ => panic!("To joint lock a node it must be declared as a joint")
        }
    }
}


pub trait PipelineRecipe<I: Sharable, O: Sharable> { // allow to save and standardize macro-scale components that you dont wan tto repeatedly redefine
    fn construct<FI: Sharable, FO: Sharable>(origin_node: PipelineNode<FI, I>, construction_queue: ConstructionQueue) -> PipelineNode<O, FO>;
}