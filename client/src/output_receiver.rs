use std::fmt::Display;

use futures_util::{Stream, StreamExt};
use tokio::{select, sync::broadcast};
use tokio_tungstenite::tungstenite::Message;

pub struct OutputReceiver<R> {
    read: R,
    shutdown: broadcast::Receiver<()>,
}

impl<R> OutputReceiver<R> {
    pub fn new(read: R, shutdown: broadcast::Receiver<()>) -> Self {
        OutputReceiver { read, shutdown }
    }

    pub async fn run<E>(mut self)
    where
        R: Stream<Item = Result<Message, E>> + Unpin,
        E: Display,
    {
        // Read from websocket and print to stdout
        while let Some(msg) = self.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received: {}", text);
                }
                Ok(other) => {
                    println!("Received non-text message: {:?}", other);
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    break;
                }
            }
        }
    }

    async fn next<E>(&mut self) -> Option<Result<Message, E>>
    where
        R: Stream<Item = Result<Message, E>> + Unpin,
    {
        select! {
            _ = self.shutdown.recv() => None,
            msg = self.read.next() => msg
        }
    }
}
