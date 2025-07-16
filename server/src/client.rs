use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};

/// Representation of a chat client in the server code.
/// 
/// We receive messages from the client and broadcast them to all other clients.
pub struct Client {
    stream: tokio::net::TcpStream,
    tx: broadcast::Sender<String>,
    rx: broadcast::Receiver<String>,
}

impl Client {
    pub fn new(
        stream: tokio::net::TcpStream,
        tx: broadcast::Sender<String>,
        rx: broadcast::Receiver<String>,
    ) -> Self {
        Self { stream, tx, rx }
    }

    pub async fn run(self) {
        let ws_stream = match accept_async(self.stream).await {
            Ok(ws) => ws,
            Err(e) => {
                eprintln!("WebSocket handshake failed: {}", e);
                return;
            }
        };
        println!("New WebSocket connection");

        let (mut write, mut read) = ws_stream.split();

        // Task to forward broadcast messages to this client
        let mut rx = self.rx;
        let broadcast_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if write.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
        });

        // Read messages from this client and broadcast them
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Optionally, filter out empty messages
                    if !text.trim().is_empty() {
                        let _ = self.tx.send(text.to_string());
                    }
                }
                Ok(Message::Close(_)) => break,
                _ => {}
            }
        }

        // Optionally, wait for the broadcast task to finish
        let _ = broadcast_task.await;
    }
}
