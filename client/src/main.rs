use std::fmt::Display;

use futures_util::{SinkExt, Stream, StreamExt};
use tokio::{
    io::{AsyncBufReadExt, BufReader, stdin},
    select,
    signal::ctrl_c,
    sync::broadcast,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let (send_shutdown, recv_shutdown) = broadcast::channel::<()>(1);
    let (ws_stream, _) = connect_async("ws://127.0.0.1:8080")
        .await
        .expect("Failed to connect");

    let (write, read) = ws_stream.split();

    let mut shutdown_input = recv_shutdown.resubscribe();
    let stdin_actor = InputSender::new(write, async move {
        shutdown_input
            .recv()
            .await
            .expect("shutdown sender must outlive receiver");
    });
    let send_input = tokio::spawn(async move {
        stdin_actor.run().await;
    });

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
    let _ = send_input.await;
    let _ = receive_output.await;
}

// Actor responsible for reading lines from stdin and sending them to the server
struct InputSender<W, S>
where
    W: SinkExt<Message> + Unpin + Send + 'static,
{
    write: W,
    shutdown: S,
}

impl<W, S> InputSender<W, S>
where
    W: SinkExt<Message> + Unpin + Send + 'static,
    S: Future<Output = ()>,
{
    fn new(write: W, shutdown: S) -> Self {
        Self { write, shutdown }
    }

    async fn run(mut self) {
        select! {
            () = self.shutdown => (),
            () = Self::consume_input_stream(&mut self.write) => (),
        }
        // We could give the server a probper good bye here
    }

    async fn consume_input_stream(write: &mut W) {
        let mut lines = BufReader::new(stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if write.send(Message::Text(line.into())).await.is_err() {
                eprintln!("Could not send message")
            }
        }
    }
}

struct OutputReceiver<R> {
    read: R,
    shutdown: broadcast::Receiver<()>,
}

impl<R> OutputReceiver<R> {
    pub fn new(read: R, shutdown: broadcast::Receiver<()>) -> Self {
        OutputReceiver { read, shutdown }
    }

    pub async fn run<E>(mut self)
    where
        R: Stream<Item = Result<Message, E>> + Unpin,
        E: Display,
    {
        // Read from websocket and print to stdout
        while let Some(msg) = self.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received: {}", text);
                }
                Ok(other) => {
                    println!("Received non-text message: {:?}", other);
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    break;
                }
            }
        }
    }

    async fn next<E>(&mut self) -> Option<Result<Message, E>>
    where
        R: Stream<Item = Result<Message, E>> + Unpin,
    {
        select! {
            _ = self.shutdown.recv() => None,
            msg = self.read.next() => msg
        }
    }
}
