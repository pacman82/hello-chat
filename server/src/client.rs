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
    send_handle: Option<tokio::task::JoinHandle<()>>,
}

impl Client {
    pub fn new(
        stream: tokio::net::TcpStream,
        tx: broadcast::Sender<String>,
        rx: broadcast::Receiver<String>,
    ) -> Self {
        Self { stream, tx, rx, send_handle: None }
    }

    pub async fn run(mut self) {
        let ws_stream = match accept_async(self.stream).await {
            Ok(ws) => ws,
            Err(e) => {
                eprintln!("WebSocket handshake failed: {}", e);
                return;
            }
        };
        println!("New WebSocket connection");

        let (mut write, mut read) = ws_stream.split();

        // Spawn the SendActor and hold its handle
        let tx = self.tx.clone();
        let send_handle = tokio::spawn(async move {
            SendActor::new(read, tx).run().await;
        });
        self.send_handle = Some(send_handle);

        // Task to forward broadcast messages to this client (remains inline for now)
        let mut rx = self.rx;
        let broadcast_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if write.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
        });

        // Optionally, wait for the broadcast task to finish
        let _ = broadcast_task.await;
        // Optionally, wait for the send_handle to finish
        if let Some(handle) = self.send_handle {
            let _ = handle.await;
        }
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