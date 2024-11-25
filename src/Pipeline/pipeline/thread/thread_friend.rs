use std::sync::mpsc;
use super::thread_diagnostics::{BaseThreadDiagnostic, PipelineError, PipelineThreadState};


pub struct PipelineThreadFriend {
    message_receiver: mpsc::Receiver<BaseThreadDiagnostic>,
    message_sender: mpsc::Sender<PipelineThreadState>,
}

impl PipelineThreadFriend {
    pub fn new(receiver: mpsc::Receiver<BaseThreadDiagnostic>, sender: mpsc::Sender<PipelineThreadState>) -> PipelineThreadFriend {
        PipelineThreadFriend {
            message_receiver: receiver,
            message_sender: sender,
        }
    }

    pub fn push_state(&mut self, new_state: PipelineThreadState) -> Result<(), mpsc::SendError<PipelineThreadState>> {
        self.message_sender.send(new_state)
    }

    pub fn receive_state(&self) -> BaseThreadDiagnostic {
        self.message_receiver.recv().unwrap()
    }
}


