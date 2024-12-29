mod Pipeline;
use std::thread;
use std::time;
use std::sync::{Arc, Mutex};
use async_std::channel;
pub use Pipeline::pipeline::pipeline;
pub use Pipeline::manager::orchestration;
pub use Pipeline::node::prototype::{PipelineStep, PipelineNodeGeneric, PipelineNode};
pub use crate::Pipeline::node::messages::{Sink, Source};
use crate::Pipeline::node::buffer::{ScalarToVectorAdapter, VectorToScalarAdapter};


#[cfg(test)]
mod NodeTests {
    use Pipeline::node::{buffer::ScalarToVectorAdapter, messages::create_node_connection, prototype::PipelineNode};

    use super::*;

    #[test]
    fn prototype_test() {
        dbg!("TEST!");
        let test_step: Box<PipelineStep<u8>> = Box::new(|a: u8| a * 2);
        let mut new_node: PipelineNode<u8> = PipelineNode::new(test_step);
        let (mut input_send, input_receive) = create_node_connection::<u8>();
        let (output_send, mut output_receive) = create_node_connection::<u8>();

        let runflag = Arc::new(Mutex::new(true));
        let runflag_clone = runflag.clone();
        new_node.set_input(Box::new(input_receive));
        new_node.set_output(Box::new(output_send));

        let sender_thread = thread::spawn(move || {
            let mut data = vec![1,2,3,4,5,6,7,8,9,0];

            while data.len() > 0 {
                //dbg!("Something!");
                let point = data.pop().unwrap();
                let _ = input_send.send(point);
                //thread::sleep(time::Duration::from_millis(10));
            }
        });

        let opt_thread = thread::spawn(move || {
            while *runflag.lock().unwrap() {
                //dbg!("Something2!");
                new_node.call();
                //thread::sleep(time::Duration::from_millis(10));
            }
        });

        let receiver_thread: thread::JoinHandle<Vec<u8>> = thread::spawn(move || {
            let mut data_vec = Vec::<u8>::new();

            while data_vec.len() < 10 {
                let result: Result<u8, std::sync::mpsc::RecvError> = output_receive.recv();

                match result {
                    Ok(val) => {data_vec.push(val);}
                    Err(error) => {dbg!("{}", error);}
                }

                //thread::sleep(time::Duration::from_millis(10));
            }

            let mut runflag = runflag_clone.lock().unwrap();
            *runflag = false;

            data_vec
        });

        let _ = sender_thread.join();
        let _ = opt_thread.join();
        let data: Vec<u8> = receiver_thread.join().unwrap();
        let compared = vec![0,18,16,14,12,10,8,6,4,2];

        //vec![1,2,3,4,5,6,7,8,9,0]
        assert!(data == compared);
    }

    #[test]
    fn scalar_vector_test() {
        let (mut input_send, input_receive) = create_node_connection::<u8>();
        let (output_send, mut output_receive) = create_node_connection::<Vec<u8>>();

        let mut new_scalar_vector = ScalarToVectorAdapter::<u8>::new(input_receive, output_send, 10);

        let mut data: Vec<u8> = vec![1,2,3,4,5,6,7,8,9,0];

        while data.len() > 0 {
            let received = data.remove(0);
            
            let _ = input_send.send(received);
            new_scalar_vector.call();
        }

        let new_data = output_receive.recv().unwrap();
        assert!(new_data == vec![1,2,3,4,5,6,7,8,9,0]);
    }

    #[test]
    fn vector_scalar_test() {
        let (mut input_send, input_receive) = create_node_connection::<Vec<u8>>();
        let (output_send, mut output_receive) = create_node_connection::<u8>();

        let mut new_vector_scalar = VectorToScalarAdapter::<u8>::new(input_receive, output_send, 10);
        
        let _ = input_send.send(vec![1,2,3,4,5,6,7,8,9,0]);

        let mut out_vec: Vec<u8> = Vec::new();

        let processor_thread = thread::spawn(move || {
            new_vector_scalar.call();
        });

        while out_vec.len() < 10 {
            match output_receive.recv() {
                Ok(val) => {
                    out_vec.push(val);
                }
                Err(error) => {
                    dbg!(error);
                    break;
                }
            }
        }

        processor_thread.join();

        assert!(out_vec == vec![1,2,3,4,5,6,7,8,9,0]);
    }
}

#[cfg(test)]
mod ThreadTests {
    use async_std::task;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use crate::Pipeline::{
        node::{
            prototype::{PipelineNode},
            messages::{create_node_connection, SenderWrapper, Sink, Source},
        }, 
        pipeline::{
            welder::Welder,
            node_enum::PipelineNodeEnum, thread::{
                pipeline_thread::{
                    create_thread_and_tap, ThreadTapManager},
                    thread_diagnostics::{
                        BaseThreadDiagnostic, PipelineThreadState}
                    }
                }
            };
    #[test]
    fn tap_manager() {
        let (in_tx, in_rx) = async_std::channel::bounded::<PipelineThreadState>(1); // maybe an issue to have unbounded if backups?
        let (out_tx, out_rx) = async_std::channel::bounded::<BaseThreadDiagnostic>(1);

        let state = Arc::new(std::sync::RwLock::new(PipelineThreadState::RUNNING));
        let time_taken = Arc::new(std::sync::RwLock::new(0 as f32));
        
        let manager: ThreadTapManager = ThreadTapManager::new(in_rx, out_tx, state, time_taken);

        let manager_thread = std::thread::spawn(move || {
            async_std::task::block_on(manager.start_taps()); // use tokio if want to have separate async runtime for thread
        });

        std::thread::sleep(std::time::Duration::from_millis(1000));
        let received_state = out_rx.recv_blocking().unwrap();

        {
        let received = received_state.thread_state.read().unwrap();
        assert!(*received == PipelineThreadState::RUNNING);
        }

        in_tx.send_blocking(PipelineThreadState::STOPPED);
        let received_state = out_rx.recv_blocking().unwrap();

        std::thread::sleep(std::time::Duration::from_millis(1000));
        {
        let received = received_state.thread_state.read().unwrap();
        assert!(*received == PipelineThreadState::STOPPED);
        }

        in_tx.send_blocking(PipelineThreadState::KILLED);
        manager_thread.join();
    }

    #[test]
    fn thread() {
        let mut node = PipelineNode::new(Box::new(move |x: u8| {
            x * 2
        }));

        let (mut input_send, input_receive) = create_node_connection::<u8>();
        let (output_send, mut output_receive) = create_node_connection::<u8>();
        
        
        node.set_input(Box::new(input_receive));
        node.set_output(Box::new(output_send));

        let (mut task_thread, mut friend) = create_thread_and_tap(
            PipelineNodeEnum::Scalar(node),
            "test".to_string()
        );
        let friend_arc = Arc::new(Mutex::new(friend));

        let sender_thread = thread::spawn(move || {
            let mut data = vec![1,2,3,4,5,6,7,8,9,0];

            while data.len() > 0 {
                //dbg!("Something!");
                let point = data.pop().unwrap();
                let _ = input_send.send(point);
                //thread::sleep(time::Duration::from_millis(10));
            }
        });

        let opt_thread = thread::spawn(move || {
            while *task_thread.check_state().read().unwrap() != PipelineThreadState::KILLED {
                task_thread.call();
            }
        });

        let friend_clone = friend_arc.clone();
        let receiver_thread: thread::JoinHandle<Vec<u8>> = thread::spawn(move || {
            let mut data_vec = Vec::<u8>::new();

            while data_vec.len() < 10 {
                let result: Result<u8, std::sync::mpsc::RecvError> = output_receive.recv();

                match result {
                    Ok(val) => {data_vec.push(val);}
                    Err(error) => {dbg!("{}", error);}
                }

                //thread::sleep(time::Duration::from_millis(10));
            }

            {
            task::block_on(friend_clone.lock().unwrap().push_state(PipelineThreadState::KILLED));
            }

            data_vec
        });

        {
        task::block_on(friend_arc.lock().unwrap().push_state(PipelineThreadState::RUNNING));
        }

        let _ = sender_thread.join();
        let _ = opt_thread.join();
        let data: Vec<u8> = receiver_thread.join().unwrap();
        let compared = vec![0,18,16,14,12,10,8,6,4,2];

        //vec![1,2,3,4,5,6,7,8,9,0]
        assert!(data == compared);
    }
}

#[cfg(test)]
mod PipelineTests {
    use async_std::task;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::sync::mpsc;
    use std::time::Duration;
    use num::PrimInt;
    use crate::Pipeline::node::buffer;
    use crate::Pipeline::{
        node::{
            prototype::{PipelineNode, PipelineStep},
            messages::{create_node_connection, SenderWrapper, Sink, Source},
        }, 
        pipeline::{
            pipeline::BytePipeline,
            welder::Welder,
            node_enum::PipelineNodeEnum, thread::{
                pipeline_thread::{
                    create_thread_and_tap, ThreadTapManager},
                    thread_diagnostics::{
                        BaseThreadDiagnostic, PipelineThreadState}
                    }
                }
            };
    
    #[test]
    fn welder_test() {
        let buffer_size: usize = 10;
        let mut test_welder = Welder::new(buffer_size);

        let (mut input_send, input_receive) = create_node_connection::<u8>();
        let (output_send, mut output_receive) = create_node_connection::<u8>();

        let test_step_1: Box<PipelineStep<u8>> = Box::new(|a: u8| a * 2);
        let mut node_1 = PipelineNode::new(test_step_1);
        node_1.set_input(Box::new(input_receive));
        let mut new_node_1: PipelineNodeEnum<u8> = PipelineNodeEnum::Scalar(node_1);

        let test_step_2: Box<PipelineStep<u8>> = Box::new(|a: u8| a + 10);
        let mut node_2 = PipelineNode::new(test_step_2);
        node_2.set_output(Box::new(output_send));
        let mut new_node_2: PipelineNodeEnum<u8> = PipelineNodeEnum::Scalar(node_2);

        test_welder.weld(&mut new_node_1, &mut new_node_2);
        
        input_send.send(10);
        new_node_1.call();
        new_node_2.call();
        let output = output_receive.recv().unwrap();

        assert!(output == 30);
    }

    struct TestSupplier {}
    impl Source<Vec<u8>> for TestSupplier {
        fn recv(&mut self) -> Result<Vec<u8>, mpsc::RecvError> {
            return Ok(vec![1,2,3,4,5,6,7,8,9,0]);
        }
    }

    struct TestEater {
        sender: SenderWrapper<Vec<u8>>
    }
    impl Sink<Vec<u8>> for TestEater {
        fn send(&mut self, to_send: Vec<u8>) -> Result<(), mpsc::SendError<Vec<u8>>> {
            dbg!("TO SEND {}", &to_send);
            self.sender.send(to_send);

            Ok(())
        }
    }

    impl TestEater {
        pub fn new(sender: SenderWrapper<Vec<u8>>) -> TestEater {
            TestEater {
                sender
            }
        }
    }

    #[test]
    fn pipeline_test() {
        let source_step: Box<PipelineStep<Vec<u8>>> = Box::new(|mut a: Vec<u8>| {
            let mut return_vec = Vec::new();
            for val in a.iter_mut() {
                return_vec.push(*val * 2);
            }

            //dbg!("{}", &return_vec);

            return_vec
        });

        let intermediate_step: Box<PipelineStep<u8>> = Box::new(|a: u8| {
            let returny = a + 4;
            returny
        });
        
        let sink_step: Box<PipelineStep<Vec<u8>>> = Box::new(|mut a: Vec<u8>| {
            let mut return_vec = Vec::new();
            for val in a.iter_mut() {
                return_vec.push(*val + 2);
            }

            dbg!("SILLY {}", &return_vec);

            return_vec
        });
        let mut source_node = PipelineNode::new(source_step);
        let mut sink_node = PipelineNode::new(sink_step);

        let (mut output_send, mut output_receive) = create_node_connection::<Vec<u8>>();

        source_node.set_input(Box::new(TestSupplier{}));
        sink_node.set_output(Box::new(TestEater::new(output_send)));

        let mut test_pipeline = BytePipeline::new(10, source_node, sink_node);

        test_pipeline.add_scalar_step(Box::new(intermediate_step), "test_step".to_string());
        test_pipeline.compose_threads();
        task::block_on(test_pipeline.run());
        
        let received = output_receive.recv().unwrap();
        let mut refe: Vec<u8> = vec![1,2,3,4,5,6,7,8,9,0];
        let mut transformed_refe: Vec<u8> = refe.iter().map(|&x| {
            let mut y = x;
            y *= 2;
            y += 4;
            y += 2;
            y
        }).collect();

        dbg!("{}", &received);
        assert!(received == transformed_refe);

        task::block_on(test_pipeline.end());
    }
}