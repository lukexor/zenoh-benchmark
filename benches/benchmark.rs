use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration, SamplingMode, Throughput,
};
use futures::Future;
use pprof::criterion::{Output, PProfProfiler};
use prost::bytes::Bytes;
use std::{
    cell::RefCell,
    sync::{Arc, Barrier},
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;
use zenoh_benchmark::TestMessage;

/// The number of messages to send on each benchmark iteration.
pub const NUM_MESSAGES: u64 = 1000;
/// The topic to publish/subscribe to.
pub const TOPIC: &str = "test-topic";

/// ZMQ URL to publish/subscribe to.
pub const ZMQ_URL: &str = "tcp://127.0.0.1:5556";
/// NATs server URL.
pub const NATS_URL: &str = "nats://127.0.0.1:4222";

/// The generic input passed to each benchmark. Attempts to normalize inputs to ensure fairness
/// across crates which have different message and topic inputs and conversion
/// implementations.
#[derive(Debug)]
#[must_use]
struct BenchInput<T> {
    connection: RefCell<Option<T>>,
    message: Bytes,
    topic: String,
}

impl<T> BenchInput<T> {
    /// Create a new `BenchInput` from a connection with a given message size.
    fn new(connection: T, msg_size: usize) -> Self {
        Self {
            connection: RefCell::new(Some(connection)),
            message: TestMessage::new(msg_size).into_bytes(),
            topic: TOPIC.into(),
        }
    }
}

/// Spawns a subscriber thread.
fn spawn_sub<F, Fut>(barrier: Arc<Barrier>, f: F)
where
    F: FnOnce(Arc<Barrier>) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(f(barrier));
}

async fn nats_setup(size: usize) -> BenchInput<async_nats::Client> {
    let barrier = Arc::new(Barrier::new(2));
    spawn_sub(barrier.clone(), nats_sub);

    let client = async_nats::connect(NATS_URL)
        .await
        .expect("valid nats server");

    barrier.wait();
    BenchInput::new(client, size)
}
async fn zenoh_setup(size: usize) -> BenchInput<zenoh::Session> {
    use zenoh::Wait;

    let barrier = Arc::new(Barrier::new(2));
    spawn_sub(barrier.clone(), zenoh_sub);

    let session = zenoh::open(zenoh::Config::default())
        .wait()
        .expect("unable to start publisher session");

    barrier.wait();
    BenchInput::new(session, size)
}
async fn zeromq_setup(size: usize) -> BenchInput<zeromq::PubSocket> {
    use zeromq::Socket;

    let barrier = Arc::new(Barrier::new(2));
    spawn_sub(barrier.clone(), zeromq_sub);

    let mut socket = zeromq::PubSocket::new();
    socket
        .bind(ZMQ_URL)
        .await
        .expect("unable to bind to socket");

    barrier.wait();
    BenchInput::new(socket, size)
}
async fn zmq_setup(size: usize) -> BenchInput<zmq::Socket> {
    let barrier = Arc::new(Barrier::new(2));
    spawn_sub(barrier.clone(), zmq_sub);

    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::PUB).unwrap();
    socket.bind(ZMQ_URL).unwrap();

    barrier.wait();
    BenchInput::new(socket, size)
}

async fn nats_teardown(client: async_nats::Client) {
    client.flush().await.expect("flushed");
}
async fn zenoh_teardown(session: zenoh::Session) {
    use zenoh::Wait;

    session.close().wait().expect("closed session");
}
async fn zeromq_teardown(socket: zeromq::PubSocket) {
    use zeromq::Socket;

    socket.close().await;
}
async fn zmq_teardown(socket: zmq::Socket) {
    socket.disconnect(ZMQ_URL).expect("disconnected");
}

async fn nats_sub(barrier: Arc<Barrier>) {
    use futures::StreamExt;

    let client = async_nats::connect(NATS_URL)
        .await
        .expect("failed to connect");
    let mut sub = client.subscribe(TOPIC).await.expect("failed to subscribe");

    barrier.wait();
    while let Some(msg) = sub.next().await {
        black_box(msg);
    }
}
async fn zenoh_sub(barrier: Arc<Barrier>) {
    let session = zenoh::open(zenoh::Config::default())
        .await
        .expect("unable to start sub session");
    let subscriber = session
        .declare_subscriber(TOPIC)
        .await
        .expect("unable to create subscriber");

    barrier.wait();
    while let Ok(msg) = subscriber.recv_async().await {
        black_box(msg);
    }
}
async fn zeromq_sub(barrier: Arc<Barrier>) {
    use zeromq::{Socket, SocketRecv};

    let mut socket = zeromq::SubSocket::new();
    socket.connect(ZMQ_URL).await.expect("failed to connect");
    socket.subscribe(TOPIC).await.expect("failed to subscribe");

    barrier.wait();
    while let Ok(msg) = socket.recv().await {
        black_box(msg);
    }
}
async fn zmq_sub(barrier: Arc<Barrier>) {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::SUB).unwrap();
    socket.set_rcvtimeo(20_000).unwrap(); // Ensure this thread quits eventually
    socket.connect(ZMQ_URL).expect("failed to connect");
    socket
        .set_subscribe(TOPIC.as_bytes())
        .expect("failed to subscribe");

    barrier.wait();
    while let Ok(msg) = socket.recv_msg(0) {
        black_box(msg);
    }
}

async fn nats_pub(client: async_nats::Client, topic: String, message: Bytes) -> async_nats::Client {
    client
        .publish(topic, message)
        .await
        .expect("valid send message");
    client
}
async fn zenoh_pub(session: zenoh::Session, topic: String, message: Bytes) -> zenoh::Session {
    session
        .put(topic, message)
        .await
        .expect("unable to publish message");
    session
}
async fn zeromq_pub(
    mut socket: zeromq::PubSocket,
    topic: String,
    message: Bytes,
) -> zeromq::PubSocket {
    use zeromq::SocketSend;

    // ZMQ encodes subjects with a SNDMORE flag in a multi-part message
    socket
        .send(topic.into())
        .await
        .expect("unable to publish over socket");
    socket
        .send(message.into())
        .await
        .expect("unable to publish over socket");
    socket
}
async fn zmq_pub(socket: zmq::Socket, topic: String, message: Bytes) -> zmq::Socket {
    // ZMQ encodes subjects with a SNDMORE flag in a multi-part message
    socket
        .send_multipart([topic.as_bytes(), &*message], 0)
        .expect("unable to publish over socket");
    socket
}

pub fn benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().expect("valid runtime");

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("Pub-Sub");
    group.plot_config(plot_config);
    group.sampling_mode(SamplingMode::Flat);

    let message_sizes = [32, 128, 512, 2048, 8192, 32768, 131072, 524288];
    for msg_size in message_sizes {
        group.throughput(Throughput::Bytes(NUM_MESSAGES * msg_size as u64));
        // group.throughput(Throughput::Elements(NUM_MESSAGES));

        // Bench each implementation by setting up the initial input and subscriber thread.
        {
            let input = rt.block_on(nats_setup(msg_size));
            group.bench_with_input(BenchmarkId::new("nats", msg_size), &input, |b, input| {
                b.iter_custom(|iters| transport_bench(&rt, iters, input, nats_pub));
            });
            rt.block_on(nats_teardown(
                input.connection.take().expect("valid connection"),
            ));
        }
        {
            let input = rt.block_on(zenoh_setup(msg_size));
            group.bench_with_input(BenchmarkId::new("zenoh", msg_size), &input, |b, input| {
                b.iter_custom(|iters| transport_bench(&rt, iters, input, zenoh_pub));
            });
            rt.block_on(zenoh_teardown(
                input.connection.take().expect("valid connection"),
            ));
        }
        {
            let input = rt.block_on(zeromq_setup(msg_size));
            group.bench_with_input(BenchmarkId::new("zeromq", msg_size), &input, |b, input| {
                b.iter_custom(|iters| transport_bench(&rt, iters, input, zeromq_pub));
            });
            rt.block_on(zeromq_teardown(
                input.connection.take().expect("valid connection"),
            ));
        }
        {
            let input = rt.block_on(zmq_setup(msg_size));
            group.bench_with_input(BenchmarkId::new("zmq", msg_size), &input, |b, input| {
                b.iter_custom(|iters| transport_bench(&rt, iters, input, zmq_pub));
            });
            rt.block_on(zmq_teardown(
                input.connection.take().expect("valid connection"),
            ));
        }
    }

    // Generic transport bencher to work around various issues with criterion, futures and lifetimes with regards to
    // the transport connections.
    //
    // We want a single connection for the entire set of criterion iterations, otherwise we may run
    // out of file descriptors with a large iteration number because internally, criterion collects
    // into an array, so connections must be created outside of the benchmark funciton. Secondly,
    // some connection types can't be held across `await` points due to not being Clone and thus
    // have lifetime issues.
    //
    // Instead we take an `Option` out and swap it in and out it on every iteration.
    //
    // Various attempts to use `iter_batched`, `iter_batched_ref`, and `iter_custom` all ran into
    // issues with setup, connection lifetimes, or file descriptor exhaustion.
    fn transport_bench<T, R, RFut>(
        rt: &Runtime,
        iters: u64,
        input: &BenchInput<T>,
        routine: R,
    ) -> Duration
    where
        R: Fn(T, String, Bytes) -> RFut,
        RFut: Future<Output = T>,
    {
        rt.block_on(async {
            let start = Instant::now();
            for _ in 0..iters {
                for _ in 0..NUM_MESSAGES {
                    let connection = input.connection.take().unwrap();
                    let connection = black_box(
                        routine(connection, input.topic.clone(), input.message.clone()).await,
                    );
                    input.connection.replace(Some(connection));
                }
            }
            start.elapsed()
        })
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = benchmarks
);
criterion_main!(benches);
