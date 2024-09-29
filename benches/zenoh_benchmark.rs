use std::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use prost::Message;
use tokio::runtime::Runtime;
use zenoh::{self, config::Config, Session, Wait};
use log::*;

const NUM_MESSAGES: u64 = 1000;
const DURATION: u64 = 30;
const INPUT: &str = "Input";

#[derive(Message)]
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

async fn start_sub() {
    let session = zenoh::open(Config::default())
        .await
        .expect("Unable to start sub session");
    let subscriber = session
        .declare_subscriber(INPUT.to_string())
        .await
        .expect("Unable to create subscriber");
    while let Ok(sample) = subscriber.recv_async().await {
        trace!("Received: {:?}", sample);
    }
}

async fn send_pub(session: Session, num_messages: u64) {
    for _ in 0..num_messages {
        session
            .put(
                INPUT.to_string(),
                Message::encode_to_vec(&test_message("nats pubsub".into())),
            )
            .await
            .expect("Unable to publish message");
    }
}

pub fn pubsub_benchmark(c: &mut Criterion) {
    env_logger::init();

    let runtime = Runtime::new().expect("Unable to start tokio Runtime");

    let session = zenoh::open(zenoh::Config::default())
        .wait()
        .expect("Unable to start publisher session");

    runtime.spawn(start_sub());

    std::thread::sleep(Duration::from_millis(1000));

    runtime.block_on(send_pub(session.clone(), NUM_MESSAGES));

    let mut group = c.benchmark_group("Pub-Sub");
    group.throughput(Throughput::Elements(NUM_MESSAGES));
    group.measurement_time(Duration::from_secs(DURATION));

    // TODO: Figure out why opening and closing the Zenoh Session
    // within the benchmark_function causes this to malfunction
    let session = zenoh::open(zenoh::Config::default())
        .wait()
        .expect("Unable to start publisher session");
    group.bench_function("zenoh", |b| {
        b.to_async(&runtime)
            .iter(|| async { 
                // let session = zenoh::open(zenoh::Config::default())
                //     .await
                //     .expect("Unable to start publisher session");
                send_pub(session.clone(), NUM_MESSAGES).await;
                // session.close().await.expect("Unable to close sesion");
            });
    });

    session.close().wait().expect("Unable to close sesion");

    group.finish();
}

criterion_group!(benches, pubsub_benchmark);
criterion_main!(benches);
