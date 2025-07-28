use std::fmt::Display;

use crate::{input_sender::InputSender, output_receiver::OutputReceiver};
use futures_util::{Sink, StreamExt};
use tokio::{sync::watch, task::JoinHandle};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};

pub struct Client {
    send_input: JoinHandle<()>,
    receive_output: JoinHandle<()>,
}

impl Client {
    pub async fn from_websocket(
        request: impl IntoClientRequest + Unpin,
        shutdown: watch::Receiver<bool>,
        user_name: String,
    ) -> Self {
        let (ws_stream, _) = connect_async(request).await.expect("Failed to connect");
        let (write, read) = ws_stream.split();

        Self::new(write, read, shutdown, user_name)
    }

    pub fn new<W, R, E>(
        write: W,
        read: R,
        shutdown: watch::Receiver<bool>,
        user_name: String,
    ) -> Self
    where
        W: Sink<Message> + Unpin + Send + 'static,
        R: futures_util::Stream<Item = Result<Message, E>> + Unpin + Send + 'static,
        E: Display,
    {
        let input_sender = InputSender::new(write, shutdown.clone(), user_name);
        let send_input = tokio::spawn(async move {
            input_sender.run().await;
        });

        let output_receiver = OutputReceiver::new(read, shutdown);
        let receive_output = tokio::spawn(async move {
            output_receiver.run().await;
        });

        Self {
            send_input,
            receive_output,
        }
    }

    pub async fn wait_for_shutdown(self) {
        let _ = self.send_input.await;
        let _ = self.receive_output.await;
    }
}
