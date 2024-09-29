use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use log::*;
use prost::Message;
use std::time::Duration;
use tokio::runtime::Runtime;
use zenoh::{self, config::Config, Session, Wait};
use zenoh_benchmark::{test_message, DURATION, INPUT, NUM_MESSAGES};

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

    runtime.spawn(start_sub());

    std::thread::sleep(Duration::from_millis(1000));

    let mut group = c.benchmark_group("Pub-Sub");
    group.throughput(Throughput::Elements(NUM_MESSAGES));
    group.measurement_time(Duration::from_secs(DURATION));

    let session = zenoh::open(zenoh::Config::default())
        .wait()
        .expect("Unable to start publisher session");
    group.bench_function("zenoh", |b| {
        b.to_async(&runtime).iter(|| async {
            send_pub(session.clone(), NUM_MESSAGES).await;
        });
    });

    session.close().wait().expect("Unable to close sesion");

    group.finish();
}

criterion_group!(benches, pubsub_benchmark);
criterion_main!(benches);
