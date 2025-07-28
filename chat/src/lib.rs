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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_payload_serialization_roundtrip() {
        let original = MessagePayload {
            content: "Hello, world!".to_string(),
        };
        let serialized = original.serialize();
        let deserialized = MessagePayload::deserialize(&serialized);
        assert_eq!(original.content, deserialized.content);
    }
}
