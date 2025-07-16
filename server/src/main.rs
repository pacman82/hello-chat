mod client;

use tokio::{net::TcpListener, sync::broadcast};

use crate::client::Client;

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
        tokio::spawn(async move {
            let _client = Client::new(stream, tx, rx).await;
        });
    }
}
