use std::sync::mpsc::{self, Receiver};

use crate::Pipeline::buffer::BufferType;


pub trait Source<DataType: BufferType> {
    fn recv(&mut self) -> Result<DataType, mpsc::RecvError>;
}

pub trait Sink<DataType: BufferType> {
    fn send(&mut self, to_send: DataType) -> Result<(), mpsc::SendError<DataType>>;
}

pub struct ReceiverWrapper<DataType: BufferType> {
    receiver: mpsc::Receiver<DataType>
}

pub struct SenderWrapper<DataType: BufferType> {
    sender: mpsc::Sender<DataType>
}

impl<DataType: BufferType> Source<DataType> for ReceiverWrapper<DataType> {
    fn recv(&mut self) -> Result<DataType, mpsc::RecvError> {
        return self.receiver.recv();
    }
}

impl<DataType: BufferType>  Sink<DataType> for SenderWrapper<DataType> {
    fn send(&mut self, to_send: DataType) -> Result<(), mpsc::SendError<DataType>> {
        return self.sender.send(to_send);
    }
}

pub fn create_node_connection<T: BufferType> () -> (SenderWrapper<T>, ReceiverWrapper<T>) {
    let (tx, rx) = mpsc::channel();
    (
        SenderWrapper::<T> {
            sender: tx
        },
        ReceiverWrapper::<T> {
            receiver: rx
        }
    )
}


