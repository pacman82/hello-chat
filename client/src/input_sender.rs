use chat::MessagePayload;
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
    user_name: String,
}

impl<W> InputSender<W>
where
    W: Sink<Message> + Unpin + Send + 'static,
{
    pub fn new(write: W, shutdown: watch::Receiver<bool>) -> Self {
        Self {
            write,
            shutdown,
            user_name: "Joe".to_string(),
        }
    }

    pub async fn run(mut self) {
        let InputSender {
            write,
            shutdown,
            user_name,
        } = &mut self;
        select! {
            _ = async { shutdown.changed().await.ok() } => (),
            () = InputSender::consume_input_stream(write, user_name) => (),
        }
        // We could give the server a proper good bye here
    }

    async fn consume_input_stream(write: &mut W, user_name: &String)
    where
        W: Sink<Message> + Unpin + Send + 'static,
    {
        let mut lines = BufReader::new(stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let payload = MessagePayload {
                sender: user_name.clone(),
                content: line,
            };
            if write
                .send(Message::Text(payload.serialize().into()))
                .await
                .is_err()
            {
                eprintln!("Could not send message")
            }
        }
    }
}
