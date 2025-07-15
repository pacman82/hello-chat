use futures_util::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncBufReadExt, BufReader, stdin},
    sync::mpsc,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("ws://127.0.0.1:8080")
        .await
        .expect("Failed to connect");

    let (mut write, mut read) = ws_stream.split();

    // Channel to send lines from stdin to the websocket writer task
    let (tx, mut rx) = mpsc::channel::<String>(16);

    // Task to read from stdin
    let tx_stdin = tx.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx_stdin.send(line).await.is_err() {
                break;
            }
        }
    });

    // Task to write to websocket
    let write_task = tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            if write.send(Message::Text(line.into())).await.is_err() {
                break;
            }
        }
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
