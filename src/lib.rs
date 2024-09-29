pub const NATS_URL: &str = "nats://127.0.0.1:4222";
pub const NUM_MESSAGES: u64 = 1000;
pub const DURATION: u64 = 30;
pub const INPUT: &str = "Input";

#[derive(prost::Message)]
#[must_use]
pub struct TestMessage {
    #[prost(string, tag = "1")]
    id: String,
    #[prost(string, tag = "2")]
    request_id: String,
    #[prost(string, tag = "3")]
    correlation_id: String,
    #[prost(string, tag = "4")]
    source_id: String,
    #[prost(string, tag = "5")]
    target_id: String,
    #[prost(bytes = "vec", tag = "6")]
    content: Vec<u8>,
}

// Create a test message for publishing/requesting
pub fn test_message(source_id: String) -> TestMessage {
    TestMessage {
        id: "73aea97e-af46-4e54-bae4-c33a10466f99".into(),
        request_id: "bc5272c9-37ed-4ba9-afcc-5c18ed8abe31".into(),
        correlation_id: "a09bdd8c-18d9-4c4a-80ca-707f225e4ce3".into(),
        source_id,
        target_id: "test".into(),
        content: vec![0; 182],
    }
}
