use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct MessagePayload {
    pub sender: String,
    pub content: String,
}

impl MessagePayload {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize MessagePayload")
    }

    pub fn deserialize(text: &str) -> MessagePayload {
        serde_json::from_str(text).expect("Failed to deserialize MessagePayload")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_payload_serialization_roundtrip() {
        let original = MessagePayload {
            sender: "alice".to_string(),
            content: "Hello, world!".to_string(),
        };
        let serialized = original.serialize();
        let deserialized = MessagePayload::deserialize(&serialized);
        assert_eq!(original, deserialized);
    }
}
