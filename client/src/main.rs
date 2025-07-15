use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt, BufReader, stdin};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("ws://127.0.0.1:8080")
        .await
        .expect("Failed to connect");

    let (write, mut read) = ws_stream.split();

    // Spawn the stdin actor, which owns the write sink
    let stdin_actor = InputActor::new(write);
    let write_task = tokio::spawn(async move {
        stdin_actor.run().await;
    });

    // Read from websocket and print to stdout
    while let Some(msg) = read.next().await {
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

    let _ = write_task.await;
}

// Actor responsible for reading lines from stdin and sending them to the server
struct InputActor<W>
where
    W: SinkExt<Message> + Unpin + Send + 'static,
{
    write: W,
}

impl<W> InputActor<W>
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
