use futures_util::{Sink, SinkExt};
use tokio::{
    io::{AsyncBufReadExt, BufReader, stdin},
    select,
    sync::watch,
};
use tokio_tungstenite::tungstenite::Message;

// Actor responsible for reading lines from stdin and sending them to the server
pub struct InputSender<W>
where
    W: Sink<Message> + Unpin + Send + 'static,
{
    write: W,
    shutdown: watch::Receiver<bool>,
}

impl<W> InputSender<W>
where
    W: Sink<Message> + Unpin + Send + 'static,
{
    pub fn new(write: W, shutdown: watch::Receiver<bool>) -> Self {
        Self { write, shutdown }
    }

    pub async fn run(mut self) {
        select! {
            _ = async {
                while !*self.shutdown.borrow() {
                    self.shutdown.changed().await.ok();
                }
            } => (),
            () = Self::consume_input_stream(&mut self.write) => (),
        }
        // We could give the server a proper good bye here
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
