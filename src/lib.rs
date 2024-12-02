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
    use crate::Pipeline::{pipeline::thread::{pipeline_thread::ThreadTapManager, thread_diagnostics::{BaseThreadDiagnostic, PipelineThreadState}}};
    #[test]
    fn tap_manager() {
        let (in_tx, in_rx) = async_std::channel::bounded::<PipelineThreadState>(1); // maybe an issue to have unbounded if backups?
        let (out_tx, out_rx) = async_std::channel::bounded::<BaseThreadDiagnostic>(1);

        let state = std::sync::Arc::new(std::sync::RwLock::new(PipelineThreadState::RUNNING));
        let time_taken = std::sync::Arc::new(std::sync::RwLock::new(0 as f32));
        
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

        {
        let received = received_state.thread_state.read().unwrap();
        dbg!("HERE! {}", &*received);
        assert!(*received == PipelineThreadState::STOPPED);
        }

        in_tx.send_blocking(PipelineThreadState::KILLED);
        manager_thread.join();
    }
}