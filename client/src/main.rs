use std::fmt::Display;

use futures_util::{SinkExt, Stream, StreamExt};
use tokio::io::{AsyncBufReadExt, BufReader, stdin};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("ws://127.0.0.1:8080")
        .await
        .expect("Failed to connect");

    let (write, read) = ws_stream.split();

    let stdin_actor = InputSender::new(write);
    let send_input = tokio::spawn(async move {
        stdin_actor.run().await;
    });

    let output_receiver = OutputReceiver::new(read);
    let _receive_output = tokio::spawn(async move { output_receiver.run().await });

    let _ = send_input.await;
}

// Actor responsible for reading lines from stdin and sending them to the server
struct InputSender<W>
where
    W: SinkExt<Message> + Unpin + Send + 'static,
{
    write: W,
}

impl<W> InputSender<W>
where
    W: SinkExt<Message> + Unpin + Send + 'static,
{
    fn new(write: W) -> Self {
        Self { write }
    }

    async fn run(mut self) {
        let mut lines = BufReader::new(stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if self.write.send(Message::Text(line.into())).await.is_err() {
                break;
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
