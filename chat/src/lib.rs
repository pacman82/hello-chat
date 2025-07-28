#[derive(Clone)]
pub struct MessagePayload {
    pub content: String,
}

impl MessagePayload {
    pub fn serialize(&self) -> &str {
        &self.content
    }

    pub fn deserialize(text: &str) -> MessagePayload {
        MessagePayload {
            content: text.to_string(),
        }
    }
}
