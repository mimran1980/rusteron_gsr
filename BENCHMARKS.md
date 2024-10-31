# Benchmarks: Aeron Java vs. Rusteron Rust IPC Throughput

This benchmark document compares the throughput performance of the existing Aeron `EmbeddedExclusiveIpcThroughput` benchmark in Java with the equivalent implementation in Rust using `rusteron_client`. The benchmark used for Java is `EmbeddedExclusiveIpcThroughput`, while the Rust implementation is ported to `rusteron-client/examples/embedded_exclusive_ipc_throughput.rs`.

**Warning**: Writing good benchmarks is always challenging, and many factors can influence the results. Until these benchmarks are verified by multiple people and in different environments, take these results with a pinch of salt.

## Benchmarks

### Exclusive IPC Throughput

This section details the exclusive IPC throughput benchmark, comparing Java and Rust implementations.

## Running the Benchmarks

### Java IPC Throughput Benchmark
The Java benchmark can be executed using the following `just` command:

```sh
just benchmark-java-ipc-throughput
```

### Rust IPC Throughput Benchmark
The Rust benchmark involves two steps. First, run the Aeron Media Driver for Rust, and then run the IPC throughput benchmark:

```sh
just run-aeron-media-driver-rust
just benchmark-rust-ipc-throughput
```

## Benchmark Results

### Java IPC Throughput Results
The results obtained by running the Java `EmbeddedExclusiveIpcThroughput` benchmark on an M1 MacBook are as follows:

```
Duration 1007ms - 29,084,302 messages - 930,697,664 payload bytes
Duration 1001ms - 28,269,764 messages - 904,632,448 payload bytes
Duration 1001ms - 27,716,075 messages - 886,914,400 payload bytes
Duration 1002ms - 28,217,443 messages - 902,958,176 payload bytes
Duration 1001ms - 26,952,002 messages - 862,464,064 payload bytes
Duration 1001ms - 29,162,348 messages - 933,195,136 payload bytes
Duration 1001ms - 28,432,625 messages - 909,844,000 payload bytes
Duration 1002ms - 29,374,684 messages - 939,989,888 payload bytes
Duration 1001ms - 27,872,288 messages - 891,913,216 payload bytes
```

### Rust IPC Throughput Results
The results obtained by running the Rust port of the Java benchmark using Rusteron on the same M1 MacBook are as follows:

```
Throughput: 36,859,281 msgs/sec, 1,179,496,981 bytes/sec
Throughput: 37,699,149 msgs/sec, 1,206,372,773 bytes/sec
Throughput: 36,795,387 msgs/sec, 1,177,452,397 bytes/sec
Throughput: 36,535,444 msgs/sec, 1,169,134,222 bytes/sec
Throughput: 36,445,890 msgs/sec, 1,166,268,494 bytes/sec
Throughput: 36,736,759 msgs/sec, 1,175,576,282 bytes/sec
Throughput: 37,174,194 msgs/sec, 1,189,574,210 bytes/sec
Throughput: 37,490,359 msgs/sec, 1,199,691,476 bytes/sec
Throughput: 37,328,495 msgs/sec, 1,194,511,837 bytes/sec
Throughput: 35,525,506 msgs/sec, 1,136,816,205 bytes/sec
Throughput: 36,739,436 msgs/sec, 1,175,661,956 bytes/sec
Throughput: 36,449,411 msgs/sec, 1,166,381,156 bytes/sec
Throughput: 35,312,234 msgs/sec, 1,129,991,486 bytes/sec
Throughput: 37,361,933 msgs/sec, 1,195,581,866 bytes/sec
Throughput: 36,977,467 msgs/sec, 1,183,278,956 bytes/sec
Throughput: 36,397,079 msgs/sec, 1,164,706,525 bytes/sec
Throughput: 36,540,587 msgs/sec, 1,169,298,774 bytes/sec
Throughput: 36,122,391 msgs/sec, 1,155,916,511 bytes/sec
Throughput: 37,562,581 msgs/sec, 1,202,002,593 bytes/sec
Throughput: 36,723,374 msgs/sec, 1,175,147,956 bytes/sec
Throughput: 36,403,125 msgs/sec, 1,164,900,012 bytes/sec
Throughput: 36,450,019 msgs/sec, 1,166,400,612 bytes/sec
Throughput: 38,015,912 msgs/sec, 1,216,509,192 bytes/sec
Throughput: 37,447,615 msgs/sec, 1,198,323,687 bytes/sec
Throughput: 37,126,740 msgs/sec, 1,188,055,668 bytes/sec
```

## Summary
From the results above, we observe that the Rust implementation consistently achieves higher throughput compared to the Java version:
- **Java**: Average throughput of approximately 28 million messages per second.
- **Rust**: Average throughput of approximately 36-38 million messages per second.

The Rust implementation shows a noticeable improvement in throughput, with an approximate **30% improvement** over the Java version. This is higher than expected, as Java low latency applications are usually around 10-20% slower compared to implementations in C++ or Rust. This could indicate that there may be a flaw in the benchmark, and the results should be interpreted with caution. Such an improvement is expected when comparing Java low latency implementations with those written in languages like C++ or Rust, which allow more control over system-level resources and typically have lower runtime overhead.

The Rust implementation shows a noticeable improvement in throughput, thanks to its system-level control and efficient use of resources.

## Next Steps
If you have suggestions for further optimizations or would like to contribute to the Rust port (`rusteron`), feel free to open an issue or a pull request on GitHub. We're always looking for ways to push the boundaries of performance!
