use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};

/// Representation of a chat client in the server code.
/// 
/// We receive messages from the client and broadcast them to all other clients.
pub struct Client {
    send_handle: tokio::task::JoinHandle<()>,
}

impl Client {
    pub async fn new(
        stream: tokio::net::TcpStream,
        tx: broadcast::Sender<String>,
        mut rx: broadcast::Receiver<String>,
    ) -> Self {
        let ws_stream = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(e) => {
                eprintln!("WebSocket handshake failed: {}", e);
                panic!("WebSocket handshake failed");
            }
        };
        println!("New WebSocket connection");

        let (mut write, mut read) = ws_stream.split();

        // Spawn the SendActor and hold its handle
        let send_handle = tokio::spawn(async move {
            // Spawn broadcast receiver task inline
            let broadcast_task = tokio::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    if write.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
            });
            SendActor::new(read, tx.clone()).run().await;
            let _ = broadcast_task.await;
        });

        Self { send_handle }
    }
}

/// Actor responsible for reading messages from the websocket and sending them to the broadcast
/// channel
pub struct SendActor<R>
where
    R: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    read: R,
    tx: broadcast::Sender<String>,
}

impl<R> SendActor<R>
where
    R: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    pub fn new(read: R, tx: broadcast::Sender<String>) -> Self {
        Self { read, tx }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if !text.trim().is_empty() {
                        let _ = self.tx.send(text.to_string());
                    }
                }
                Ok(Message::Close(_)) => break,
                _ => {}
            }
        }
    }
}