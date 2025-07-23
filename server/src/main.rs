mod client;

use tokio::{
    net::TcpListener,
    signal::ctrl_c,
    sync::{broadcast, watch},
};

use crate::client::Client;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    let (shutdown_send, shutdown_recv) = watch::channel(false);
    println!("WebSocket server listening on {}", addr);

    let handle = tokio::spawn(async move {
        relay_messages_between_clients(listener, shutdown_recv).await;
    });

    ctrl_c().await.expect("Failed to listen for ctrl_c");
    shutdown_send.send(true).unwrap();

    eprintln!("Shutting down server...");

    // handle.await.unwrap();
}

async fn relay_messages_between_clients(listener: TcpListener, shutdown: watch::Receiver<bool>) {
    // Create a broadcast channel for all clients
    let (tx, _) = broadcast::channel::<String>(100);

    let mut clients = Vec::new();

    while let Ok((stream, _)) = listener.accept().await {
        let tx = tx.clone();
        let client = Client::new(stream, tx).await;
        clients.push(client);
    }
}
