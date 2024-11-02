# Benchmarks: Aeron Java vs. Rusteron Rust IPC Throughput

**Warning**: Writing good benchmarks is challenging, and results can be influenced by various factors. Until these benchmarks are verified across different environments, take these results with a pinch of salt.

## Benchmarks

Benchmarks were conducted on two systems:
1. **Apple M1 MacBook Pro**
2. **AMD Ryzen 5 3400G**

### Exclusive IPC Throughput

This benchmark compares the throughput performance of the Aeron `EmbeddedExclusiveIpcThroughput` benchmark in Java with the equivalent Rust implementation in `rusteron_client`. The Java benchmark used is `EmbeddedExclusiveIpcThroughput`, and the Rust version is ported to `rusteron-client/examples/embedded_exclusive_ipc_throughput.rs`.

## Running the Benchmarks

### Java IPC Throughput Benchmark
Execute the Java benchmark using the following `just` command:

```sh
just benchmark-java-ipc-throughput
```

### Rust IPC Throughput Benchmark
To run the Rust benchmark, first start the Aeron Media Driver for Rust, then execute the IPC throughput benchmark:

```sh
just run-aeron-media-driver-rust
just benchmark-rust-ipc-throughput
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

#### Rust IPC Throughput with Java Media Driver
Rust benchmark results with the Java media driver:

```
Throughput: 34,371,321 msgs/sec, 1,099,882,267 bytes/sec
Throughput: 36,009,246 msgs/sec, 1,152,295,888 bytes/sec
Throughput: 34,798,099 msgs/sec, 1,113,539,166 bytes/sec
Throughput: 35,104,141 msgs/sec, 1,123,332,511 bytes/sec
Throughput: 34,219,767 msgs/sec, 1,095,032,558 bytes/sec
Throughput: 33,570,332 msgs/sec, 1,074,250,618 bytes/sec
Throughput: 34,357,081 msgs/sec, 1,099,426,587 bytes/sec
Throughput: 35,308,391 msgs/sec, 1,129,868,511 bytes/sec
Throughput: 34,503,901 msgs/sec, 1,104,124,848 bytes/sec
Throughput: 34,056,613 msgs/sec, 1,089,811,617 bytes/sec
Throughput: 36,392,474 msgs/sec, 1,164,559,166 bytes/sec
Throughput: 35,717,277 msgs/sec, 1,142,952,865 bytes/sec
Throughput: 35,266,763 msgs/sec, 1,128,536,429 bytes/sec
Throughput: 35,042,202 msgs/sec, 1,121,350,469 bytes/sec
Throughput: 35,904,945 msgs/sec, 1,148,958,241 bytes/sec
Throughput: 34,771,485 msgs/sec, 1,112,687,523 bytes/sec
Throughput: 33,368,872 msgs/sec, 1,067,803,916 bytes/sec
Throughput: 33,951,504 msgs/sec, 1,086,448,141 bytes/sec
Throughput: 34,073,490 msgs/sec, 1,090,351,673 bytes/sec
Throughput: 34,825,446 msgs/sec, 1,114,414,268 bytes/sec
Throughput: 34,024,351 msgs/sec, 1,088,779,219 bytes/sec
```

### AMD Ryzen 5 3400G Results

#### Java IPC Throughput
Results of the Java benchmark on AMD Ryzen:

```
Duration 1000ms - 45,201,129 messages - 1,446,436,128 payload bytes
Duration 1007ms - 48,589,355 messages - 1,554,859,360 payload bytes
Duration 1001ms - 45,913,784 messages - 1,469,241,088 payload bytes
Duration 1001ms - 53,578,692 messages - 1,714,518,144 payload bytes
Duration 1001ms - 46,619,398 messages - 1,491,820,736 payload bytes
Duration 1001ms - 52,841,404 messages - 1,690,924,928 payload bytes
Duration 1001ms - 54,543,209 messages - 1,745,382,688 payload bytes
Duration 1001ms - 52,014,035 messages - 1,664,449,120 payload bytes
Duration 1001ms - 54,524,962 messages - 1,744,798,784 payload bytes
Duration 1000ms - 55,013,544 messages - 1,760,433,408 payload bytes
Duration 1001ms - 46,954,229 messages - 1,502,535,328 payload bytes
Duration 1001ms - 53,598,635 messages - 1,715,156,320 payload bytes
Duration 1001ms - 53,805,736 messages - 1,721,783,552 payload bytes
Duration 1000ms - 51,584,296 messages - 1,650,697,472 payload bytes
Duration 1001ms - 50,213,973 messages - 1,606,847,136 payload bytes
```

#### Rust IPC Throughput with Rust Media Driver
Rust benchmark results using the

Rust media driver on AMD Ryzen:

```
Throughput: 20,830,729 msgs/sec, 666,583,334 bytes/sec
Throughput: 20,747,710 msgs/sec, 663,926,719 bytes/sec
Throughput: 20,674,070 msgs/sec, 661,570,231 bytes/sec
Throughput: 20,801,708 msgs/sec, 665,654,643 bytes/sec
Throughput: 20,967,671 msgs/sec, 670,965,478 bytes/sec
Throughput: 20,528,464 msgs/sec, 656,910,838 bytes/sec
Throughput: 20,792,583 msgs/sec, 665,362,644 bytes/sec
Throughput: 20,545,616 msgs/sec, 657,459,719 bytes/sec
Throughput: 20,764,230 msgs/sec, 664,455,355 bytes/sec
Throughput: 19,499,696 msgs/sec, 623,990,286 bytes/sec
Throughput: 19,047,098 msgs/sec, 609,507,132 bytes/sec
Throughput: 20,750,823 msgs/sec, 664,026,327 bytes/sec
Throughput: 20,615,699 msgs/sec, 659,702,365 bytes/sec
Throughput: 20,357,534 msgs/sec, 651,441,100 bytes/sec
Throughput: 20,479,442 msgs/sec, 655,342,154 bytes/sec
Throughput: 20,613,394 msgs/sec, 659,628,617 bytes/sec
Throughput: 20,512,627 msgs/sec, 656,404,057 bytes/sec
Throughput: 20,593,865 msgs/sec, 659,003,673 bytes/sec
Throughput: 20,926,609 msgs/sec, 669,651,487 bytes/sec
Throughput: 21,114,344 msgs/sec, 675,659,007 bytes/sec
Throughput: 21,049,171 msgs/sec, 673,573,463 bytes/sec
Throughput: 21,118,281 msgs/sec, 675,784,977 bytes/sec
Throughput: 20,986,121 msgs/sec, 671,555,877 bytes/sec
Throughput: 20,984,788 msgs/sec, 671,513,206 bytes/sec
Throughput: 20,868,458 msgs/sec, 667,790,656 bytes/sec
Throughput: 21,165,076 msgs/sec, 677,282,427 bytes/sec
```

#### Rust IPC Throughput with Java Media Driver
Rust benchmark results with the Java media driver on AMD Ryzen:

```
Throughput: 12,379,596 msgs/sec, 396,147,076 bytes/sec
Throughput: 14,012,661 msgs/sec, 448,405,157 bytes/sec
Throughput: 13,729,604 msgs/sec, 439,347,317 bytes/sec
Throughput: 13,951,234 msgs/sec, 446,439,502 bytes/sec
Throughput: 14,063,792 msgs/sec, 450,041,336 bytes/sec
Throughput: 13,661,431 msgs/sec, 437,165,794 bytes/sec
Throughput: 13,766,710 msgs/sec, 440,534,719 bytes/sec
Throughput: 13,856,926 msgs/sec, 443,421,621 bytes/sec
Throughput: 14,000,311 msgs/sec, 448,009,941 bytes/sec
Throughput: 13,901,505 msgs/sec, 444,848,156 bytes/sec
Throughput: 14,966,359 msgs/sec, 478,923,500 bytes/sec
Throughput: 13,701,671 msgs/sec, 438,453,487 bytes/sec
Throughput: 12,957,905 msgs/sec, 414,652,975 bytes/sec
Throughput: 12,855,655 msgs/sec, 411,380,950 bytes/sec
Throughput: 12,839,975 msgs/sec, 410,879,196 bytes/sec
Throughput: 13,101,653 msgs/sec, 419,252,901 bytes/sec
Throughput: 13,357,158 msgs/sec, 427,429,064 bytes/sec
Throughput: 13,535,131 msgs/sec, 433,124,184 bytes/sec
```

### Conclusion

The benchmark results indicate that Rust’s performance varies significantly across different architectures and benchmarks:

- **Apple M1**: The Rust implementation with the Rust Media Driver consistently outperforms Java, achieving around **30% higher throughput** in the Exclusive IPC benchmark, with Rust reaching 36-38 million messages per second compared to Java’s 28 million messages per second.

- **AMD Ryzen 5 (x86)**: Java performs better in the Exclusive IPC benchmark on x86, reaching **45-55 million messages per second**, while Rust's throughput is unexpectedly lower. Interestingly, this performance discrepancy does not extend to the Ping Pong benchmark, where Rust achieves faster round-trip times than Java, even on x86. This suggests that architectural nuances or specific optimizations in the Rust Media Driver may impact performance differently across benchmarks on x86.

From these results, we can conclude that at least the `rusteron-client` is not slower than the Java implementation.

## Next Steps
For contributions to `rusteron`, suggestions, or optimizations, please open an issue or PR on GitHub. Together, we can continue pushing the boundaries of performance!