use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use log::*;
use prost::Message;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use zenoh_benchmark::{test_message, DURATION, INPUT, NUM_MESSAGES};
use zeromq::{Socket, SocketRecv, SubSocket, PubSocket, SocketSend};

async fn start_sub() {
    info!("Before opening subscriber socker");
    let mut socket = SubSocket::new();
    socket
        .connect("tcp://127.0.0.1:5556")
        .await
        .expect("Failed to connect");
    socket.subscribe(INPUT).await.expect("Failed to subscribe");
    info!("After opening subscriber socket");
    while let Ok(msg) = socket.recv().await {
        trace!("Received: {:?}", msg);
    }
}

async fn send_pub(pub_socket: Arc<Mutex<PubSocket>>, num_messages: u64) {
    let mut pub_socket = pub_socket.lock().unwrap();
    for _ in 0..num_messages {
        pub_socket
            .send(Message::encode_to_vec(&test_message("nats pubsub".into())).into())
            .await
            .expect("Unable to publish over socket");
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

    info!("Before opening publisher socket");
    let mut socket = PubSocket::new();
    runtime.block_on(socket.bind("tcp://127.0.0.1:5556")).expect("Unable to bind to socket");
    let socket = Arc::new(Mutex::new(socket));
    info!("After opening publisher socket");
    group.bench_function("zeromq", |b| {
        b.to_async(&runtime).iter(|| async {
            send_pub(socket.clone(), NUM_MESSAGES).await;
        });
    });

    group.finish();
}

criterion_group!(benches, pubsub_benchmark);
criterion_main!(benches);
