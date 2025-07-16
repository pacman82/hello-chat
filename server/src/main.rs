use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("WebSocket server listening on {}", addr);

    // Create a broadcast channel for all clients
    let (tx, _) = broadcast::channel::<String>(100);

    while let Ok((stream, _)) = listener.accept().await {
        let tx = tx.clone();
        let rx = tx.subscribe();
        tokio::spawn(handle_connection(stream, tx, rx));
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    tx: broadcast::Sender<String>,
    mut rx: broadcast::Receiver<String>,
) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake failed: {}", e);
            return;
        }
    };
    println!("New WebSocket connection");

    let (mut write, mut read) = ws_stream.split();

    // Task to forward broadcast messages to this client
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
                    let _ = tx.send(text.to_string());
                }
            }
            Ok(Message::Close(_)) => break,
            _ => {}
        }
    }

    // Optionally, wait for the broadcast task to finish
    let _ = broadcast_task.await;
}
