# Benchmarks: Aeron Java vs. Rusteron Rust IPC Throughput

**Warning**: Writing good benchmarks is challenging, and results can be influenced by various factors. Until these benchmarks are verified across different environments, take these results with a pinch of salt.

## Benchmarks

Benchmarks were conducted on two systems:
1. **Apple M1 MacBook Pro**
2. **AMD EPYC 7R32 48-Core Processor**

### Exclusive IPC Throughput

This benchmark compares the throughput performance of the Aeron `EmbeddedExclusiveIpcThroughput` benchmark in Java with the equivalent Rust implementation in `rusteron_client`. The Java benchmark used is `EmbeddedExclusiveIpcThroughput`, and the Rust version is ported to `rusteron-client/examples/embedded_exclusive_ipc_throughput.rs`.

## Running the Benchmarks

### Java IPC Throughput Benchmark
Execute the Java benchmark using the following `just` command:

```sh
just benchmark-ipc-throughput-java
```

### Rust IPC Throughput Benchmark
To run the Rust benchmark, first start the Aeron Media Driver for Rust, then execute the IPC throughput benchmark:

```sh
just run-aeron-media-driver-rust
just benchmark-ipc-throughput-rust
```

## Benchmark Results

### M1 MacBook Pro Results

#### Java IPC Throughput
Results of the Java benchmark on an M1 MacBook:

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

#### Rust IPC Throughput with Rust Media Driver
Rust benchmark results using the Rust media driver:

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

### AMD EPYC 7R32 48-Core Processor Results

**Note**: For these benchmarks, the Aeron directory is set to `/dev/shm`.

#### Java IPC Throughput
Results of the Java benchmark on the AMD EPYC 7R32 48-Core Processor:

```
Duration 1000ms - 11,209,199 messages - 358,694,368 payload bytes
Duration 1004ms - 11,012,081 messages - 352,386,592 payload bytes
Duration 1001ms - 10,937,523 messages - 350,000,736 payload bytes
Duration 1000ms - 11,061,422 messages - 353,965,504 payload bytes
Duration 1001ms - 11,171,619 messages - 357,491,808 payload bytes
Duration 1000ms - 11,153,734 messages - 356,919,488 payload bytes
Duration 1001ms - 11,018,808 messages - 352,601,856 payload bytes
Duration 1001ms - 10,937,833 messages - 350,010,656 payload bytes
Duration 1000ms - 10,871,451 messages - 347,886,432 payload bytes
Duration 1001ms - 11,158,473 messages - 357,071,136 payload bytes
Duration 1000ms - 11,187,960 messages - 358,014,720 payload bytes
Duration 1001ms - 10,723,554 messages - 343,153,728 payload bytes
```

#### Rust IPC Throughput with Rust Media Driver
Rust benchmark results using the Rust media driver:

```
Throughput: 38,655,705 msgs/sec, 1,236,982,553 bytes/sec
Throughput: 39,360,449 msgs/sec, 1,259,534,357 bytes/sec
Throughput: 38,413,688 msgs/sec, 1,229,238,010 bytes/sec
Throughput: 38,180,298 msgs/sec, 1,221,769,540 bytes/sec
Throughput: 38,959,568 msgs/sec, 1,246,706,163 bytes/sec
Throughput: 39,463,926 msgs/sec, 1,262,845,637 bytes/sec
Throughput: 38,960,897 msgs/sec, 1,246,748,714 bytes/sec
Throughput: 38,788,058 msgs/sec, 1,241,217,869 bytes/sec
Throughput: 37,881,413 msgs/sec, 1,212,205,227 bytes/sec
Throughput: 38,339,134 msgs/sec, 1,226,852,280 bytes/sec
Throughput: 39,242,001 msgs/sec, 1,255,744,022 bytes/sec
```

### Ping Pong Benchmark (ran on AMD EPYC 7R32 - UDP)

This section details the Ping Pong benchmark, which is based on the Aeron `EmbeddedPingPong` sample. The benchmark involves a warm-up phase of 100,000 messages followed by the actual benchmark of 10,000,000 messages, with a message size length of 32 bytes. Regular publishers were used, not exclusive publishers. The channels were `aeron:udp?endpoint=localhost:20123` and `aeron:udp?endpoint=localhost:20124`.

#### Rust Ping Pong Results with Rust Media Driver
The results of the Rust benchmark using the Rust media driver are as follows:

```
PING: pong publisher aeron:udp?endpoint=localhost:20124 1003
PING: ping subscriber aeron:udp?endpoint=localhost:20123 1002

Histogram of RTT latencies:
# of samples: 10000000
min: 7.824µs
50th percentile: 9.327µs
99th percentile: 12.799µs
99.9th percentile: 18.847µs
99.99th percentile: 26.175µs
max: 138.936319ms
avg: 9.918µs
```

#### Java Ping Pong Results
The results of the Java Ping Pong benchmark are as follows:

```
Publishing Ping at aeron:udp?endpoint=localhost:20123 on stream id 1002
Subscribing Ping at aeron:udp?endpoint=localhost:20123 on stream id 1002
Subscribing Pong at aeron:udp?endpoint=localhost:20124 on stream id 1003
Publishing Pong at aeron:udp?endpoint=localhost:20124 on stream id 1003
Message payload length of 32 bytes
Using exclusive publications: false
Waiting for new image from Pong...
Warming up... 10 iterations of 10,000 messages
Pinging 10,000,000 messages
Histogram of RTT latencies in microseconds.
       Value     Percentile TotalCount 1/(1-Percentile)

       8.879 0.500000000000    5030832           2.00
      11.967 0.990625000000    9906648         106.67
      16.767 0.999023437500    9990356        1024.00
      24.527 0.999902343750    9999026       10240.00
#[Mean    =        9.290, StdDeviation   =      225.704]
#[Max     =   650641.407, Total count    =     10000000]
#[Buckets =           24, SubBuckets     =         2048]
```

### Conclusion

The benchmark results indicate that Rust's performance on the AMD EPYC 7R32 48-Core Processor significantly outperforms Java in the Exclusive IPC Throughput benchmark:

- **AMD EPYC 7R32 48-Core Processor**: The Rust implementation with the Rust Media Driver achieves around **38-39 million messages per second**, while the Java implementation reaches approximately **11 million messages per second** in the Exclusive IPC benchmark. This shows that Rust's throughput is about **3.5 times higher** than Java's on this hardware configuration.

- In the Ping Pong benchmark using UDP, Rust also demonstrates comparable latency to Java, with an average RTT latency of **9.918µs** for Rust and **9.290µs** for Java. The minimal difference indicates similar performance between Rust and Java in this test.

- **Apple M1**: On the Apple M1 MacBook Pro, the Rust implementation with the Rust Media Driver consistently outperforms Java, achieving around **30% higher throughput** in the Exclusive IPC benchmark, with Rust reaching **36-38 million messages per second** compared to Java’s **28 million messages per second**.

From these results, we can conclude that the `rusteron-client` not only matches but can significantly outperform the Java implementation, particularly in the Exclusive IPC Throughput benchmark on high-core-count processors like the AMD EPYC 7R32. Additionally, using `/dev/shm` for the Aeron directory can further enhance performance by utilizing in-memory file systems.

## Next Steps

For contributions to `rusteron`, suggestions, or optimizations, please open an issue or PR on GitHub. Together, we can continue pushing the boundaries of performance!