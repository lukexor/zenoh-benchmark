use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use prost::Message;
use std::time::Duration;
use tokio::runtime::Runtime;

const NATS_URL: &str = "nats://127.0.0.1:4222";
const NUM_MESSAGES: u64 = 1000;
const INPUT: &str = "Input";

#[derive(prost::Message)]
#[must_use]
struct TestMessage {
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
fn test_message(source_id: String) -> TestMessage {
    TestMessage {
        id: "73aea97e-af46-4e54-bae4-c33a10466f99".into(),
        request_id: "bc5272c9-37ed-4ba9-afcc-5c18ed8abe31".into(),
        correlation_id: "a09bdd8c-18d9-4c4a-80ca-707f225e4ce3".into(),
        source_id,
        target_id: "test".into(),
        content: vec![0; 182],
    }
}

async fn nats_pubsub(client: async_nats::Client, num_messages: u64) {
    for _ in 0..num_messages {
        client
            .publish(
                INPUT.to_string(),
                Message::encode_to_vec(&test_message("nats pubsub".into())).into(),
            )
            .await
            .expect("valid send message");
    }
}

fn pubsub_benchmark(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    env_logger::init();

    let mut group = c.benchmark_group("Pub-Sub");
    group.throughput(Throughput::Elements(NUM_MESSAGES));
    group.measurement_time(Duration::from_secs(30));

    let connect_opts = async_nats::ConnectOptions::new()
        .retry_on_initial_connect()
        .no_echo()
        .name("nats_pubsub");
    let client = runtime
        .block_on(connect_opts.connect(NATS_URL))
        .expect("valid nats server");
    group.bench_function("nats", |b| {
        b.to_async(&runtime)
            .iter(|| nats_pubsub(client.clone(), NUM_MESSAGES));
    });

    group.finish();
}

criterion_group!(benches, pubsub_benchmark);
criterion_main!(benches);
