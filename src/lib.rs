use prost::{bytes::Bytes, Message};

#[derive(prost::Message)]
#[must_use]
pub struct TestMessage {
    #[prost(bytes = "vec", tag = "1")]
    pub content: Vec<u8>,
}

impl TestMessage {
    pub fn new(size: usize) -> Self {
        Self {
            content: vec![0; size],
        }
    }

    pub fn into_bytes(&self) -> Bytes {
        Message::encode_to_vec(self).into()
    }
}
