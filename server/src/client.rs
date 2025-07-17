use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::broadcast, task::JoinHandle};
use tokio_tungstenite::{accept_async, tungstenite::Message};

/// Representation of a chat client in the server code.
///
/// We receive messages from the client and broadcast them to all other clients.
pub struct Client {
    _inbound_handle: JoinHandle<()>,
    _outbound_handle: JoinHandle<()>,
}

impl Client {
    pub async fn new(stream: TcpStream, tx: broadcast::Sender<String>) -> Self {
        let rx = tx.subscribe();
        let ws_stream = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(e) => {
                eprintln!("WebSocket handshake failed: {}", e);
                panic!("WebSocket handshake failed");
            }
        };
        println!("New WebSocket connection");

        let (write, read) = ws_stream.split();

        // Spawn the inbound broadcaster (client -> broadcast channel)
        let inbound_handle = tokio::spawn(async move {
            InboundToBroadcast::new(read, tx.clone()).run().await;
        });

        // Spawn the outbound broadcaster (broadcast channel -> client)
        let outbound_handle = tokio::spawn(async move {
            BroadcastToOutbound::new(rx, write).run().await;
        });

        // Optionally, you could join both handles here or store both
        Self {
            _inbound_handle: inbound_handle,
            _outbound_handle: outbound_handle,
        }
    }
}

/// Actor responsible for reading messages from the websocket and sending them to the broadcast
/// channel
pub struct InboundToBroadcast<R>
where
    R: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    read: R,
    tx: broadcast::Sender<String>,
}

impl<R> InboundToBroadcast<R>
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

/// Actor responsible for receiving broadcasted messages and writing them to the websocket
pub struct BroadcastToOutbound<W>
where
    W: SinkExt<Message> + Unpin,
{
    rx: broadcast::Receiver<String>,
    write: W,
}

impl<W> BroadcastToOutbound<W>
where
    W: SinkExt<Message> + Unpin,
{
    pub fn new(rx: broadcast::Receiver<String>, write: W) -> Self {
        Self { rx, write }
    }

    pub async fn run(mut self) {
        while let Ok(msg) = self.rx.recv().await {
            if self.write.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    }
}
