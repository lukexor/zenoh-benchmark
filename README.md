# Benchmarking Comparison of Zenoh vs NATS

## Prerequisites

In order to run the `nats_benchmark` you'll need a `nats-server`.

Follow along with the instructions [here](https://docs.nats.io/running-a-nats-service/introduction/installation#getting-the-binary-from-the-command-line) on the nats.io website to obtain a `nats-server`.

You will need to run the `nats-server` in another terminal like so:

```bash
./nats-server -a 127.0.0.1
```

where we've configured the network address to match that which the client will be publishing to in the `nats_benchmark`.

## How to run the benchmarks

If you run

```bash
cargo bench
```

then the following benchmarks will be run:

* `zenoh_benchmark`
* `nats_benchmark`
* `zeromq_benchmark`

## Interpreting the results

The output will look something like this:

```bash
$ cargo bench
   Compiling zenoh-benchmark v0.1.0 (/home/pete/client-projects/statheros/zenoh-benchmark)
    Finished `bench` profile [optimized] target(s) in 3.58s
     Running unittests src/lib.rs (target/release/deps/zenoh_benchmark-88c18ef9f884a34d)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running benches/nats_benchmark.rs (target/release/deps/nats_benchmark-de1cb48840aac2c8)
Gnuplot not found, using plotters backend
Pub-Sub/nats            time:   [885.39 µs 904.24 µs 921.21 µs]
                        thrpt:  [1.0855 Melem/s 1.1059 Melem/s 1.1294 Melem/s]
                 change:
                        time:   [+2.0586% +3.6495% +5.5822%] (p = 0.00 < 0.05)
                        thrpt:  [-5.2871% -3.5210% -2.0171%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Pub-Sub/nats_with_mutex time:   [938.37 µs 943.39 µs 948.40 µs]
                        thrpt:  [1.0544 Melem/s 1.0600 Melem/s 1.0657 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild

     Running benches/zenoh_benchmark.rs (target/release/deps/zenoh_benchmark-28a69f6aa83ae4d6)
Gnuplot not found, using plotters backend
Pub-Sub/zenoh           time:   [701.93 µs 707.28 µs 713.05 µs]
                        thrpt:  [1.4024 Melem/s 1.4139 Melem/s 1.4247 Melem/s]
                 change:
                        time:   [+8.1755% +11.431% +14.791%] (p = 0.00 < 0.05)
                        thrpt:  [-12.885% -10.258% -7.5577%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild

Pub-Sub/zenoh_with_mutex
                        time:   [738.49 µs 744.79 µs 751.16 µs]
                        thrpt:  [1.3313 Melem/s 1.3427 Melem/s 1.3541 Melem/s]
                 change:
                        time:   [-1.7068% -0.5312% +0.5319%] (p = 0.36 > 0.05)
                        thrpt:  [-0.5290% +0.5340% +1.7364%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

     Running benches/zeromq_benchmark.rs (target/release/deps/zeromq_benchmark-2a5475ca42eefb40)
Gnuplot not found, using plotters backend
Pub-Sub/zeromq          time:   [2.1148 ms 2.1183 ms 2.1225 ms]
                        thrpt:  [471.14 Kelem/s 472.09 Kelem/s 472.86 Kelem/s]
                 change:
                        time:   [-0.7575% -0.5569% -0.3521%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3533% +0.5601% +0.7632%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
```

### Time and Throughput

```bash
Pub-Sub/nats            time:   [814.58 µs 818.98 µs 823.71 µs]
                        thrpt:  [1.2140 Melem/s 1.2210 Melem/s 1.2276 Melem/s]
```

Quoting from the Criterion [docs](https://bheisler.github.io/criterion.rs/book/user_guide/command_line_output.html#time):

> This shows a confidence interval over the measured per-iteration time for this benchmark. The left and right values show the lower and upper bounds of the confidence interval respectively, while the center value shows Criterion.rs' best estimate of the time taken for each iteration of the benchmarked routine.

The same holds for the throughput.

### Outliers

Quoting again from the Criterion [docs](https://bheisler.github.io/criterion.rs/book/user_guide/command_line_output.html#detecting-outliers):

> Criterion.rs attempts to detect unusually high or low samples and reports them as outliers. A large number of outliers suggests that the benchmark results are noisy and should be viewed with appropriate skepticism. In this case, you can see that there are some samples which took much longer than normal. This might be caused by unpredictable load on the computer running the benchmarks, thread or process scheduling, or irregularities in the time taken by the code being benchmarked.

## Results from my computer

### Results excluding ZeroMQ

Running the benchmarks on my computer I arrive at:

|            | NATS           | Zenoh          | Delta        |
|------------|----------------|----------------|--------------|
| time       | 818.98 µs      | 605.26 µs      | 26% decrease |
| throughput | 1.2210 Melem/s | 1.6522 Melem/s | 35% increase |

### Results including ZeroMQ

And then if we include zeromq, we'll need to compare the NATS and Zenoh benchmarks which used an `Arc<Mutex<>>` to fairly penalize them all.

(We need to do this because it would appear that the zeromq crate's key types like `PubSocket` do not implement `Clone`)

|            | NATS with Mutex | Zenoh with Mutex | ZeroMQ with Mutex | Delta NATS -> Zenoh | Delta NATS -> ZeroMQ |
|------------|-----------------|------------------|-------------------|---------------------|----------------------|
| time       | 943.39 µs       | 744.79 µs        | 2.1183 ms         | 21% **faster**      | 124% **slower**      |
| throughput | 1.0600 Melem/s  | 1.3427 Melem/s   | 472.09 Kelem/s    | 27% **more**        | 55% **less**         |

#### Thoughts

In a real application it's unclear how often we'd want to share around a zeromq socket type, but it is noteworthy that in doing so we'd be penalized by having to go through an `Arc<Mutex<>>` or some other sync primitive.

If we can avoid it, then likely the performance would be better, but gauging by the differences between with and without `Arc<Mutec>>` for NATs and Zenoh, I don't see this appreciably improving the case for ZeroMQ.

## Modifications from original `nats.rs`

I moved initialization of the client / session for both `nats_benchmark` and `zenoh_benchmark` outside of the benchmarking loop. I feel it's more representative, as these clients / sessions would be long-lived.

I also moved the common pieces of both benchmarks, like the definition of the `TestMessage` and other `const`s used to control testing into the library to make the benchmarks cleaner and more obviously consistent.
