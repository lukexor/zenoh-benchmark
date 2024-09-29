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

both the `zenoh_benchmark` and the `nats_benchmark` will be run.

## Interpreting the results

The output will look something like this:

```bash
$ cargo bench
    Finished `bench` profile [optimized] target(s) in 0.10s
     Running unittests src/lib.rs (target/release/deps/zenoh_benchmark-f24ff03bf2b8fc1f)

running 1 test
test tests::it_works ... ignored

test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running benches/nats_benchmark.rs (target/release/deps/nats_benchmark-7d735994ee6778df)
Gnuplot not found, using plotters backend
Pub-Sub/nats            time:   [814.58 µs 818.98 µs 823.71 µs]
                        thrpt:  [1.2140 Melem/s 1.2210 Melem/s 1.2276 Melem/s]
                 change:
                        time:   [-1.7198% -0.6472% +0.4504%] (p = 0.26 > 0.05)
                        thrpt:  [-0.4483% +0.6514% +1.7499%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe

     Running benches/zenoh_benchmark.rs (target/release/deps/zenoh_benchmark-c89864ddf7788408)
Gnuplot not found, using plotters backend
Pub-Sub/zenoh           time:   [592.16 µs 605.26 µs 619.11 µs]
                        thrpt:  [1.6152 Melem/s 1.6522 Melem/s 1.6887 Melem/s]
                 change:
                        time:   [-0.4725% +1.4281% +3.4461%] (p = 0.16 > 0.05)
                        thrpt:  [-3.3313% -1.4080% +0.4747%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

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

Running the benchmarks on my computer I arrive at:

|            | NATS           | Zenoh          | Delta        |
|------------|----------------|----------------|--------------|
| time       | 818.98 µs      | 605.26 µs      | 26% decrease |
| throughput | 1.2210 Melem/s | 1.6522 Melem/s | 35% increase |

## Modifications from original `nats.rs`

I moved initialization of the client / session for both `nats_benchmark` and `zenoh_benchmark` outside of the benchmarking loop. I feel it's more representative, as these clients / sessions would be long-lived.

I also moved the common pieces of both benchmarks, like the definition of the `TestMessage` and other `const`s used to control testing into the library to make the benchmarks cleaner and more obviously consistent.
