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

    let mut clients = Vec::new();

    while let Ok((stream, _)) = listener.accept().await {
        let tx = tx.clone();
        let client = Client::new(stream, tx).await;
        clients.push(client);
    }
}
