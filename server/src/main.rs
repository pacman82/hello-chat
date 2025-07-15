use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("WebSocket server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake failed: {}", e);
            return;
        }
    };
    println!("New WebSocket connection");

    let (mut write, mut read) = ws_stream.split();
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) if text == "ping" => {
                if let Err(e) = write.send(Message::Text("pong".into())).await {
                    eprintln!("Failed to send pong: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            _ => {}
        }
    }
}
