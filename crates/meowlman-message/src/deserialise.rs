use crate::Message;

pub struct MessageDeserializer;

impl MessageDeserializer {
    pub fn deserialize(raw_message: &str) -> Result<Message, String> {
        todo!("Implement deserialization of raw email message into Message struct");
    }
}
