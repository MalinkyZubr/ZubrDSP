use std::{fmt::Debug, sync::mpsc};


pub trait Source<T: 'static + Debug + Send>: Send {
    fn recv(&mut self) -> Result<T, mpsc::RecvError>;
}

pub trait Sink<T: 'static + Debug + Send>: Send {
    fn send(&mut self, to_send: T) -> Result<(), mpsc::SendError<T>>;
}

pub struct ReceiverWrapper<T: 'static> {
    receiver: mpsc::Receiver<T>
}

pub struct SenderWrapper<T: 'static> {
    sender: mpsc::Sender<T>
}

impl<T: Send + 'static + Debug> Source<T> for ReceiverWrapper<T> {
    fn recv(&mut self) -> Result<T, mpsc::RecvError> {
        return self.receiver.recv();
    }
}

impl<T: Send + 'static + Debug>  Sink<T> for SenderWrapper<T> {
    fn send(&mut self, to_send: T) -> Result<(), mpsc::SendError<T>> {
        return self.sender.send(to_send);
    }
}

pub fn create_node_connection<T> () -> (SenderWrapper<T>, ReceiverWrapper<T>) {
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


