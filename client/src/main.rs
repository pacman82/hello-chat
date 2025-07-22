mod client;
mod input_sender;
mod output_receiver;

use tokio::{signal::ctrl_c, sync::watch};

use crate::client::Client;

#[tokio::main]
async fn main() {
    // We create a watch channel to communicate that we want to shutdown
    let (send_shutdown, recv_shutdown) = watch::channel(false);

    // This starts the actual client, which connects to the backend and receives and sends messages
    // we want to continue to do so until we terminate the client. The await is finished once we
    // start sending and receiving messages.
    let client = Client::from_websocket("ws://127.0.0.1:8080", recv_shutdown.clone()).await;

    // Wait for ctrl+c before initiating shutdown
    let _ = ctrl_c().await;
    eprintln!("Ctrl+C received");
    send_shutdown
        .send(true)
        .expect("We expect there to be two active receivers");

    // Let's be nice and not leave any detached threads. This allows our sender and receiver to
    // implement (and execute) logic for a graceful shutdown, if they should wish so.
    client.wait_for_shutdown().await;
}
