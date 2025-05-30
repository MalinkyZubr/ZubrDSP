use async_std::channel;
use super::thread_diagnostics::{BaseThreadDiagnostic, PipelineThreadState};


pub struct PipelineThreadFriend {
    message_receiver: channel::Receiver<BaseThreadDiagnostic>,
    message_sender: channel::Sender<PipelineThreadState>,
    pub id: String
}

impl PipelineThreadFriend {
    pub fn new(receiver: channel::Receiver<BaseThreadDiagnostic>, sender: channel::Sender<PipelineThreadState>, id: String) -> PipelineThreadFriend {
        PipelineThreadFriend {
            message_receiver: receiver,
            message_sender: sender,
            id
        }
    }

    pub async fn push_state(&mut self, new_state: PipelineThreadState) -> Result<(), async_std::channel::SendError<PipelineThreadState>> {
        self.message_sender.send(new_state).await
    }

    pub async fn receive_state(&self) -> BaseThreadDiagnostic {
        self.message_receiver.recv().await.unwrap()
    }
}


