mod client;
mod input_sender;
mod output_receiver;

use futures_util::StreamExt;
use tokio::{signal::ctrl_c, sync::broadcast};
use tokio_tungstenite::connect_async;

use crate::{client::Client, output_receiver::OutputReceiver};

#[tokio::main]
async fn main() {
    let (send_shutdown, recv_shutdown) = broadcast::channel::<()>(1);
    let (ws_stream, _) = connect_async("ws://127.0.0.1:8080")
        .await
        .expect("Failed to connect");

    let (write, read) = ws_stream.split();

    let client = Client::new(write, recv_shutdown.resubscribe());

    let output_receiver = OutputReceiver::new(read, recv_shutdown);
    let receive_output = tokio::spawn(async move { output_receiver.run().await });

    // Wait for ctrl+c before initiating shutdown
    let _ = ctrl_c().await;
    eprintln!("Ctrl+C received");
    send_shutdown
        .send(())
        .expect("We expect there to be two active receivers");

    // Let's be nice and not leave any detached threads. This allows our sender and receiver to
    // implement (and execute) logic for a graceful shutdown, if they should wish so.
    client.wait_for_shutdown().await;
    let _ = receive_output.await;
}
