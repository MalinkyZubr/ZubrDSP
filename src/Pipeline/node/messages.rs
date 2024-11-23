use std::sync::mpsc;


pub trait Source<T>: Send {
    fn recv(&mut self) -> Result<T, mpsc::RecvError>;
}

pub trait Sink<T>: Send{
    fn send(&mut self, to_send: T) -> Result<(), mpsc::SendError<T>>;
}

pub struct ReceiverWrapper<T> {
    receiver: mpsc::Receiver<T>
}

pub struct SenderWrapper<T> {
    sender: mpsc::Sender<T>
}

impl<T> Source<T> for ReceiverWrapper<T> {
    fn recv(&mut self) -> Result<T, mpsc::RecvError> {
        return self.receiver.recv();
    }
}

impl<T>  Sink<T> for SenderWrapper<T> {
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


