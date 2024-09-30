use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use prost::Message;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use zenoh_benchmark::{test_message, DURATION, INPUT, NATS_URL, NUM_MESSAGES};

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

async fn nats_pubsub_with_mutex(client: Arc<Mutex<async_nats::Client>>, num_messages: u64) {
    let client = client.lock().unwrap();
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

fn pubsub_benchmark_no_mutex(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    env_logger::init();

    let mut group = c.benchmark_group("Pub-Sub");
    group.throughput(Throughput::Elements(NUM_MESSAGES));
    group.measurement_time(Duration::from_secs(DURATION));

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

fn pubsub_benchmark_with_mutex(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("Pub-Sub");
    group.throughput(Throughput::Elements(NUM_MESSAGES));
    group.measurement_time(Duration::from_secs(DURATION));

    let connect_opts = async_nats::ConnectOptions::new()
        .retry_on_initial_connect()
        .no_echo()
        .name("nats_pubsub");
    let client = runtime
        .block_on(connect_opts.connect(NATS_URL))
        .expect("valid nats server");
    let client = Arc::new(Mutex::new(client));
    group.bench_function("nats_with_mutex", |b| {
        b.to_async(&runtime)
            .iter(|| nats_pubsub_with_mutex(client.clone(), NUM_MESSAGES));
    });

    group.finish();
}

criterion_group!(benches, pubsub_benchmark_no_mutex, pubsub_benchmark_with_mutex);
criterion_main!(benches);
