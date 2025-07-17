use std::{fmt::Display, future::pending};

use futures_util::{SinkExt, Stream, StreamExt};
use tokio::{
    io::{AsyncBufReadExt, BufReader, stdin},
    select,
    signal::ctrl_c,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("ws://127.0.0.1:8080")
        .await
        .expect("Failed to connect");

    let (write, read) = ws_stream.split();

    let stdin_actor = InputSender::new(write, async move {
        let _ = ctrl_c().await;
        eprintln!("Ctrl+C received")
    });
    let send_input = tokio::spawn(async move {
        stdin_actor.run().await;
    });

    let output_receiver = OutputReceiver::new(read);
    let _receive_output = tokio::spawn(async move { output_receiver.run().await });

    let _ = send_input.await;
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
}

impl<R> OutputReceiver<R> {
    pub fn new(read: R) -> Self {
        OutputReceiver { read }
    }

    pub async fn run<E>(mut self)
    where
        R: Stream<Item = Result<Message, E>> + Unpin,
        E: Display,
    {
        // Read from websocket and print to stdout
        while let Some(msg) = self.read.next().await {
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
}
