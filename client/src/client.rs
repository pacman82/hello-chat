use futures_util::Sink;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;

use crate::input_sender::InputSender;

pub struct Client {
    send_input: tokio::task::JoinHandle<()>,
}

impl Client {
    pub fn new<W>(write: W, shutdown: broadcast::Receiver<()>) -> Self
    where
        W: Sink<Message> + Unpin + Send + 'static,
    {
        let input_sender = InputSender::new(write, shutdown.resubscribe());
        let send_input = tokio::spawn(async move {
            input_sender.run().await;
        });
        Self { send_input }
    }

    pub async fn wait_for_shutdown(self) {
        let _ = self.send_input.await;
    }
}
