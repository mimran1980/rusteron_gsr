**Note:** These benchmarks are environment-sensitive. Rust IPC throughput was re-measured 2026-06-27 on an Apple M1 Pro (10-core) — see [Apple M1 Pro (re-measured)](#apple-m1-pro-10-core--re-measured-2026-06-27) below. The original M1/EPYC and Java figures are retained for comparison; rerun locally for your own hardware. Latency and settings on x86-64 Linux, and a branch comparison, are in [x86-64 Linux on Azure](#x86-64-linux-on-azure-2026-10-09).

# Aeron IPC Throughput Benchmarks: Java vs. rusteron (Rust)

**Note**: These benchmarks are early-stage and environment-sensitive. Interpret results with caution until verified across varied systems.

## Systems Tested

1. Apple M1 MacBook Pro  
2. AMD EPYC 7R32 (48-core)

## What Was Measured

We compared Aeron’s `EmbeddedExclusiveIpcThroughput` benchmark in Java with the Rust port at `rusteron-client/examples/embedded_exclusive_ipc_throughput.rs`.

## How to Run

### Java
```bash
just benchmark-ipc-throughput-java
```

### Rust

Run the driver in one terminal and the benchmark in another; the recipe sets `AERON_DIR`, so the example uses that driver instead of embedding its own.

```bash
just run-aeron-media-driver-rust        # terminal 1
just benchmark-ipc-throughput-rust      # terminal 2
```

## Results

### Apple M1 MacBook Pro

**Java**: \~27–29 million msgs/sec
**Rust**: \~35–38 million msgs/sec

**Example (Rust)**:

```
Throughput: 36,859,281 msgs/sec, 1,179,496,981 bytes/sec
...
```

### Apple M1 Pro (10-core) — re-measured 2026-06-27

Rust, 32-byte IPC, SHARED client threading + DEDICATED media driver, steady-state per-second samples.

**Rust**: \~32–51 million msgs/sec (typically \~36–40M, peak \~51M)

**Example (Rust)**:

```
Throughput: 39,248,311 msgs/sec, 1,255,945,944 bytes/sec
Throughput: 50,859,457 msgs/sec, 1,627,502,615 bytes/sec
Throughput: 36,694,513 msgs/sec, 1,174,224,425 bytes/sec
...
```

(Java was not re-measured in this run; the M1 Java figure above is a reasonable reference.)

### AMD EPYC 7R32 (48-core)

**Java**: \~10.8–11.2 million msgs/sec
**Rust**: \~38–39 million msgs/sec

**Example (Rust)**:

```
Throughput: 39,360,449 msgs/sec, 1,259,534,357 bytes/sec
...
```

Rust consistently outperformed Java by \~3.5x in this benchmark.

---

## Ping Pong Benchmark (UDP, EPYC)

* Warm-up: 100,000 messages
* Main run: 10,000,000 messages (32-byte payload)
* Channels: `aeron:udp?endpoint=localhost:20123` and `:20124`
* Regular (not exclusive) publications used.

### How to Run

```bash
# Rust (the recipe sets AERON_DIR, so it needs a running driver)
just run-aeron-media-driver-rust          # terminal 1
just benchmark-embedded-ping-pong-rust    # terminal 2 (examples/embedded_ping_pong.rs)

# Java (embeds its own driver; needs the Aeron jars, see the IPC section)
just benchmark-embedded-ping-pong-java
```

### Rust

```
avg: 9.918µs
p99: 12.799µs
max: 138.936ms
```

### Java

```
avg: 9.290µs
p99: ~12–16µs
max: 650.641ms
```

---

## Summary

| Platform             | Java (msgs/sec) | Rust (msgs/sec) | Speedup |
| -------------------- | --------------- | --------------- | ------- |
| M1 MacBook           | \~28M           | \~36–38M        | \~1.3x  |
| M1 Pro (2026-06-27)  | \~28M (ref)     | \~37M (32–51M)  | \~1.3x  |
| EPYC 7R32            | \~11M           | \~38–39M        | \~3.5x  |

* Rust's `rusteron-client` shows strong throughput advantages, especially on high-core servers.
* Ping Pong (UDP) latencies are comparable between Rust and Java.
* Using `/dev/shm` for the Aeron directory improves performance (used on EPYC).

# x86-64 Linux on Azure (2026-10-09)

Latency and throughput on two small x86-64 VMs: the `main` and `improvements` branches compared, and the build, driver and system settings that matter. IPC and loopback UDP only. The README's [Tuning on x86-64 Linux](./README.md#tuning-on-x86-64-linux) summarises the settings.

## Machines

Both Azure VMs run Debian 13 (kernel 6.12), rustc 1.95.0, GCC 14 and JDK 21, with 16 GiB of memory, THP `enabled=always` and `/dev/shm` without huge pages unless a setting remounts it.

| Host | Size | CPU | Topology | L3 |
|---|---|---|---|---|
| AMD | Standard_F4as_v6 | AMD EPYC 9V74 (Zen 4) | 4 cores, no SMT | 32 MiB |
| Intel | Standard_D4s_v6 | Intel Xeon Platinum 8573C (Emerald Rapids) | 2 cores × 2 SMT threads | 260 MiB |

## Method

- **Builds.** `main` (the 0.2.10 line) and `impr` (the `improvements` branch), each built into the same out-of-tree harness with the `static` feature, fat LTO, `codegen-units=1`, Rust `target-cpu=native` and Aeron C 1.52.2 with `-O3 -DNDEBUG -funroll-loops -march=native`, unless a label says otherwise. Every group except `ps` uses one standalone C media driver binary (`media_driver`, a static `-march=native` build from `impr`, the same for every arm, so `ab` and `build` vary only the client), started fresh for every run with its own `AERON_DIR` under `/dev/shm`. Settings come from `AERON_*` environment variables.
- **rtt.** Ping offers a 32-byte message, pong echoes it with `try_claim`, and ping busy-polls for the echo and records the round trip in an HDR histogram. Exclusive publications, one message in flight. IPC: 2,000,000 round trips after a 200k warm-up. UDP: loopback unicast, 300,000 after 50k. Each run is capped at 20 s.
- **tput.** A port of Aeron's `EmbeddedExclusiveIpcThroughput`: one thread offers 32-byte messages flat out on `aeron:ipc`, the other polls them. The figure is the median of five one-second samples after a one-second warm-up.
- **ps.** A Java archive, one recording on `aeron:ipc`, and n persistent subscriptions (all LIVE) plus n plain subscriptions on the same idle stream. The figure is ns per idle poll. `thread`: the client has its conductor thread. `invoker`: the client uses the agent invoker, so every persistent subscription poll also runs a conductor duty cycle, and the plain loop calls `main_do_work()` once per round. All archive clients share one fixed control-response port.
- **Layouts.** The driver and the client conductors stay on the housekeeping CPUs; ping and pong pin themselves.
  - AMD `split`: ping and pong on their own cores, housekeeping on the other two.
  - Intel `split`: ping and pong on different cores, housekeeping on their SMT siblings.
  - Intel `smt`: ping and pong on the two threads of one core, housekeeping on the other core.
  - `unpinned`: no pinning.
- **Groups.** Arm order rotates every rep.
  - `ab` (6 reps): `main`, `impr` and `impr-aa`, the `impr` binary run again as an A/A noise floor. The driver runs DEDICATED with `noop` sender and receiver threads, so on Intel two spinning threads sit on the SMT siblings of ping and pong.
  - `build` (5 reps): driver defaults (backoff) and one change at a time against `impr`: `impr-c-x86-64` and `impr-c-x86-64-v3` (Aeron C built with `RUSTERON_C_MARCH`), `impr-rust-x86-64` (Rust `-C target-cpu=x86-64`, the default for a crate that depends on rusteron), `impr-dynamic` (dynamic linking).
  - `ipc-knobs` (5 reps): driver defaults plus one setting each: `nonsparse` (`AERON_TERM_BUFFER_SPARSE_FILE=false`), `pretouch` (nonsparse plus `AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true`), `term1m` (`AERON_IPC_TERM_BUFFER_LENGTH=1048576`), `shmhuge` (`/dev/shm` remounted `huge=always`, sparse files), `combo` (pretouch plus shmhuge), `unpinned`, and `smt` (Intel only).
  - `udp-knobs` (5 reps), loopback UDP: `dedicated-noop` (sender and receiver `noop`), `dedicated-spin` (sender and receiver `spin`), `dedicated-backoff` (all driver defaults), `shared-noop` (`AERON_THREADING_MODE=SHARED`, `AERON_SHARED_IDLE_STRATEGY=noop`), `shared-network-noop` (`SHARED_NETWORK` with a `noop` network thread), and `-smt` variants on Intel.
  - UDP rerun (12 reps, a second run on new VMs the same day): the UDP part of `ab` again, and the same with non-sparse, pre-touched logs (`ab-udp-pretouch`), back to back for every arm and rep.
- **Noise.** Each cell is the median across reps with the range across reps in brackets. A difference counts as real only if it is larger than the A/A gap for the same host and metric (`impr` against `impr-aa`), or if the rep ranges do not overlap; otherwise it is no measurable difference. `ps` had 2 reps and no A/A arm, so there only non-overlapping ranges count. On AMD, IPC p50 and p90 move in clock steps of about 9–10 ns, so a one-step p50 change is at the resolution limit and the mean is the steadier signal. `max` is one sample per rep, so it is cited only where the rep ranges do not overlap.

## Results

**main against improvements.**
- **AMD IPC:** no regression, and a small gain in typical latency for `impr`: p50 171 against 180 ns, p90 200 against 210 ns, mean 226.5 against 235 ns (`impr-aa` 225). The tails (p99 to p99.99) and throughput show no gain worth counting.
- **Intel IPC:** no difference of practical size. The gaps that pass the A/A test are at most about 1% and go both ways: p90 357 against 361 ns, mean 361.5 against 364 ns, p99.99 13503 against 13575 ns, p50 322 against 321 ns.
- **Loopback UDP, both hosts:** no regression. In the first run (6 reps), AMD p99 was 18.47 µs for `impr` against 17.90 µs for `main`, with `impr-aa`, the same binary, at 18.68 µs. The 12-rep rerun found 18.41 against 18.41 µs (A/A gap 0.09 µs), and 17.57 against 17.52 µs with pre-touched logs (A/A gap 0.10 µs). Paired by rep, neither `impr` arm was above or below `main` more often than chance at any percentile on either host (sign test p ≥ 0.10).

**IPC page faults (ipc-knobs).**
- **The default tail fits first-touch page faults.** A new publication writes fresh sparse tmpfs pages, so it faults once per 4 KiB, here every 64th 64-byte frame. Removing the faults (`pretouch`, `combo`, or `term1m` once its small log has wrapped) or making them 512× rarer (`shmhuge`) cut AMD p99 from 3025 ns to 221–240 ns, p99.9 by about 3× and p99.99 by 5.6–7.6×. On Intel only huge pages (`shmhuge`, `combo`) took p99 below 400 ns; `pretouch` and `term1m` left it at about 1.25 µs. p50 did not move on AMD and fell 4–5 ns on Intel with huge pages (289–290 against 294 ns).
- **`nonsparse` alone** trims the tail but does not remove it: AMD p99 2755 ns, with p99.9 worse than base.
- **`shmhuge` without pre-touch** adds rare stalls: max 213 µs on AMD and 523 µs on Intel, against 22 µs and 48 µs in base. `combo` keeps the gains without them.
- **`term1m`** gave the lowest AMD tail medians, level with `combo`, raised Intel throughput by 18% and lowered AMD's by 0.9%. On Intel it left p99.9 at 9.90 µs (base 9.87 µs) and raised p90 from 317 to 329 ns.
- **Intel p99.9.** It sat at about 9.9 µs in `base`, `build` and `term1m` (split layout) and in `unpinned`, fell to 1.8–3.3 µs with `pretouch`, `shmhuge` or `combo`, and was 2.1 µs in the `smt` layout. Its cause is not established. Intel p99.99 medians stayed at or above 10.1 µs in every configuration.
- **Throughput does not show the faults**, because a flat-out publisher wraps its log during warm-up. Judge fault settings by latency.

**Driver threads.**
- **Spinning sender and receiver threads on IPC** (`ab` against `build`, same `impr` binary; the two groups ran one after the other, not interleaved). On Intel, where they sit on the SMT siblings of ping and pong, they cost p50 322 against 294 ns, p90 357 against 316 ns, p99 2745 against 2457 ns, max 376 against 40 µs and 38% of throughput (47.2M against 76.1M msgs/s), but p99.9 fell from 9.85 to 3.07 µs. On AMD, where they have cores of their own, only the max got worse (331 against 18 µs).
- **UDP with backoff** (driver defaults) was the worst configuration on AMD at every statistic and on Intel from p90 up. Its reps split into fast and slow ones: on AMD 4 of 5 had p50 near 103 µs (those hit the 20 s cap with 191k–276k samples) and one matched `noop`; on Intel p90 was slow in 3 of 5. On Intel it added 31–36 µs from p90 to p99.9 and 49 µs at p99.99.
- **UDP on Intel:** `shared-noop` was best at p50, p90, p99 and the mean, and tied `shared-network-noop` at p99.9 and p99.99. Against `dedicated-noop`: p50 −10.5%, p99.9 −27%, p99.99 −57%.
- **UDP on AMD:** a trade-off. `dedicated-noop` had the best mean and tied `dedicated-spin` at p50, p90 and p99; `shared-noop` was better at p99.9 (23.6 against 29.1 µs) and p99.99 but cost 1.6 µs at p50 and 4.3 µs at p99.
- **`spin` against `noop`** for the sender and receiver: no consistent benefit. The mean was about 2.5% higher with `spin` on both hosts; AMD p99.9 was 2% lower and p99.99 higher.

**Layouts and pinning.**
- **Intel `smt` layout for IPC:** p50 106 against 294 ns, p99.9 2.1 against 9.9 µs, throughput −11%. The layout changes two things at once: ping and pong share a core, and the housekeeping threads leave their siblings.
- **Intel `smt` layout for UDP through the driver:** no help. With `dedicated-noop` the differences were small and went both ways; with `shared-network-noop` it was worse at every percentile up to p99.9.
- **Unpinned:** AMD throughput −4% and p50 one 9 ns step higher; Intel throughput spread from 71.9M to 94.0M across reps, and p99 rose 5% (2387 against 2269 ns).

**Build settings (build).**
- **AMD:** the all-native build had the lowest throughput: C `x86-64` +1.6%, C `x86-64-v3` +1.7%, Rust `x86-64` +1.3%, with ranges that do not overlap. At p50 the native build was one clock step faster (171 against 180 ns) in 13 of 15 comparisons, and the mean moved by 1–2 ns. No setting wins on both.
- **Intel:** no consistent effect. Dynamic linking had a higher p99.99 (14.62 against 14.26 µs, ranges disjoint) but a lower p99; Rust `x86-64` and dynamic linking had 3.4–3.6% lower throughput with overlapping ranges.
- **Not measured:** the combination a dependant gets by default (dynamic linking, Rust `x86-64`, C `-march=native` from source), and the precompiled libraries themselves.

**Persistent subscriptions (ps).**
- **Per-poll cost.** Nothing in a poll is shared between persistent subscriptions: each has its own archive client and control-response subscription, and every poll checks them before it reads the live image.
  - Conductor thread: an idle poll cost 24.4 and 23.4 ns on AMD and 22.6 and 22.4 ns on Intel at n = 1 and 10, against 8–12 ns for a plain subscription; at n = 100, 34.7 ns on AMD and 42.5 ns on Intel.
  - Agent invoker: every poll also runs a client conductor duty cycle: 54–62 ns at n = 1 and 10 on both hosts, and 69.3 ns on AMD and 106.8 ns on Intel at n = 100.
- **Images.** A shared fixed control-response port gave n + 1 images, not n².
- **Time to LIVE.** At n = 100, reaching LIVE took 3.6–3.8 s with the conductor thread against about 0.8 s with the invoker. A likely reason, not tested, is that the conductor thread sleeps up to 16 ms between duty cycles when idle (`AERON_CLIENT_IDLE_SLEEP_DURATION`). The figure excludes `build()`.

## AMD EPYC 9V74 (4 cores, no SMT)

ab: main against impr, IPC (rtt in ns, tput in M msgs/s):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean | tput |
|---|---|---|---|---|---|---|---|---|
| main | 6 | 180 (180–181) | 210 (210–210) | 3020 (2995–3035) | 3195 (3165–3225) | 10987 (10527–11487) | 235 (235–236) | 28.75 (28.34–28.82) |
| impr | 6 | 171 (170–171) | 200 (200–200) | 3025 (2995–3065) | 3195 (3165–3225) | 10579 (10503–11535) | 226.5 (224–228) | 28.82 (28.03–29.06) |
| impr-aa | 6 | 171 (171–180) | 200 (191–200) | 3005 (2935–3045) | 3180 (3145–3235) | 10939 (10559–11447) | 225 (224–226) | 28.78 (28.69–28.89) |

build: compile and link settings, IPC (ns, M msgs/s):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean | tput |
|---|---|---|---|---|---|---|---|---|
| impr | 5 | 171 (171–171) | 200 (191–200) | 3005 (3005–3035) | 3185 (3175–3205) | 11223 (11031–11271) | 225 (224–226) | 28.66 (28.59–28.66) |
| impr-c-x86-64 | 5 | 180 (171–180) | 200 (200–201) | 3005 (2995–3035) | 3165 (3155–3195) | 11239 (11087–11367) | 227 (225–227) | 29.12 (29.11–29.12) |
| impr-c-x86-64-v3 | 5 | 180 (180–180) | 200 (200–200) | 2995 (2995–3015) | 3175 (3165–3195) | 11231 (11159–11287) | 226 (225–227) | 29.14 (29.13–29.14) |
| impr-rust-x86-64 | 5 | 180 (171–180) | 200 (181–200) | 3045 (3025–3045) | 3205 (3185–3205) | 11303 (11103–11343) | 226 (224–227) | 29.03 (29.01–29.07) |
| impr-dynamic | 5 | 171 (171–180) | 200 (200–200) | 3015 (3005–3025) | 3185 (3165–3195) | 11263 (11119–11367) | 226 (225–226) | 28.68 (28.02–28.71) |

ipc-knobs: IPC settings (rtt in ns, max in µs, tput in M msgs/s):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean | max | tput |
|---|---|---|---|---|---|---|---|---|---|
| base | 5 | 171 (171–180) | 200 (200–200) | 3025 (3005–3035) | 3195 (3175–3205) | 11303 (11087–11439) | 226 (224–227) | 22 (20–161) | 28.66 (28.57–28.68) |
| nonsparse | 5 | 171 (171–171) | 200 (191–201) | 2755 (2755–2775) | 3557 (3435–3885) | 7263 (4547–7615) | 219 (218–219) | 20 (17–23) | 28.65 (28.63–28.66) |
| pretouch | 5 | 171 (171–180) | 200 (190–201) | 240 (230–240) | 1092 (1082–1111) | 1563 (1553–1602) | 178 (177–180) | 17 (12–33) | 28.67 (28.64–28.71) |
| term1m | 5 | 171 (171–180) | 200 (191–200) | 221 (220–221) | 1082 (1072–1092) | 1492 (1472–1512) | 178 (177–179) | 10 (10–22) | 28.39 (28.30–28.44) |
| shmhuge | 5 | 171 (171–180) | 181 (181–190) | 230 (221–230) | 1102 (1092–1132) | 2023 (2003–2073) | 176 (176–177) | 213 (207–216) | 28.75 (28.74–28.81) |
| combo | 5 | 171 (171–180) | 190 (181–190) | 221 (221–230) | 1101 (1091–1102) | 1552 (1502–1562) | 175 (174–176) | 15 (12–20) | 28.72 (28.18–28.75) |
| unpinned | 5 | 180 (171–180) | 200 (200–200) | 3015 (3005–3035) | 3195 (3155–3195) | 11503 (11279–11703) | 229 (226–230) | 64 (23–158) | 27.44 (27.01–28.11) |

ab: main against impr, loopback UDP rtt (µs):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean |
|---|---|---|---|---|---|---|---|
| main | 6 | 9.52 (9.51–9.54) | 11.25 (11.18–12.76) | 17.90 (17.87–18.56) | 29.21 (27.97–29.49) | 72.48 (64.25–138.50) | 10.22 (10.15–10.25) |
| impr | 6 | 9.54 (9.52–10.36) | 11.19 (10.94–12.78) | 18.47 (17.87–18.64) | 29.34 (27.54–29.57) | 71.04 (64.45–78.66) | 10.25 (10.18–10.35) |
| impr-aa | 6 | 10.09 (9.51–10.40) | 10.92 (10.57–11.30) | 18.68 (17.89–19.12) | 29.50 (29.04–40.26) | 74.88 (62.37–86.02) | 10.38 (10.14–10.46) |

UDP rerun (12 reps) as in ab, loopback UDP rtt (µs):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean |
|---|---|---|---|---|---|---|---|
| main | 12 | 9.55 (9.52–10.04) | 11.60 (10.10–12.92) | 18.41 (17.84–18.75) | 28.56 (26.22–81.66) | 64.75 (53.63–99.71) | 10.33 (10.20–10.51) |
| impr | 12 | 9.52 (9.51–10.38) | 12.55 (10.12–12.83) | 18.41 (17.84–18.72) | 28.85 (26.99–82.11) | 67.23 (56.09–426.50) | 10.34 (10.21–10.70) |
| impr-aa | 12 | 9.54 (9.52–10.02) | 12.39 (10.74–12.87) | 18.32 (17.87–18.67) | 28.12 (25.93–30.02) | 68.16 (52.38–83.33) | 10.32 (10.16–10.41) |

UDP rerun with pre-touched logs, loopback UDP rtt (µs):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean |
|---|---|---|---|---|---|---|---|
| main | 12 | 9.55 (9.51–10.00) | 11.26 (10.09–12.82) | 17.52 (16.30–17.71) | 27.08 (24.40–79.30) | 56.17 (50.75–93.12) | 10.15 (9.95–10.30) |
| impr | 12 | 9.68 (9.52–10.40) | 11.15 (10.46–12.76) | 17.57 (16.09–17.76) | 27.62 (25.15–28.46) | 57.05 (48.54–3170.30) | 10.18 (9.98–10.68) |
| impr-aa | 12 | 9.53 (9.52–10.64) | 11.19 (10.08–12.78) | 17.48 (16.61–17.79) | 27.55 (26.09–81.60) | 58.38 (48.48–96.25) | 10.16 (10.01–10.33) |

udp-knobs: driver threading and idle strategy, loopback UDP rtt (µs):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean |
|---|---|---|---|---|---|---|---|
| dedicated-noop | 5 | 9.53 (9.51–10.04) | 11.24 (11.15–11.28) | 18.09 (17.87–18.69) | 29.14 (28.25–29.21) | 73.86 (53.44–79.42) | 10.22 (10.15–10.46) |
| dedicated-spin | 5 | 9.61 (9.57–10.03) | 11.28 (10.81–11.34) | 18.29 (18.18–18.50) | 28.50 (27.36–29.09) | 83.90 (64.00–254.21) | 10.48 (10.39–10.52) |
| dedicated-backoff | 5 | 103.42 (9.57–106.05) | 104.13 (12.93–111.04) | 111.74 (108.73–117.25) | 127.23 (114.75–149.89) | 167.55 (126.08–257.54) | 81.65 (13.56–104.59) |
| shared-noop | 5 | 11.16 (11.13–11.21) | 12.79 (12.78–12.83) | 22.37 (22.32–22.43) | 23.63 (23.25–23.70) | 63.81 (43.04–72.83) | 11.99 (11.98–12.06) |
| shared-network-noop | 5 | 14.26 (14.10–14.70) | 14.41 (14.22–15.02) | 21.86 (21.45–22.37) | 25.02 (24.82–27.26) | 66.43 (57.25–77.89) | 14.42 (14.20–14.90) |

ps: idle poll cost, ns per poll (2 reps, unpinned), and time to LIVE:

| client | n | persistent subscription | plain subscription | time to LIVE (ms, each rep) |
|---|---|---|---|---|
| thread | 1 | 24.4 (24.4–24.4) | 12.4 (11.3–13.4) | 112, 305 |
| thread | 10 | 23.4 (23.2–23.7) | 9.7 (9.4–10.0) | 401, 401 |
| thread | 100 | 34.7 (34.6–34.7) | 12.6 (12.6–12.7) | 3780, 3643 |
| invoker | 1 | 54.2 (53.8–54.6) | 41.0 (41.0–41.1) | 17, 218 |
| invoker | 10 | 53.8 (52.6–54.9) | 12.2 (12.2–12.2) | 96, 282 |
| invoker | 100 | 69.3 (68.8–69.9) | 12.6 (12.1–13.0) | 813, 820 |

## Intel Xeon Platinum 8573C (2 cores × 2 SMT threads)

ab: main against impr, IPC (rtt in ns, tput in M msgs/s):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean | tput |
|---|---|---|---|---|---|---|---|---|
| main | 6 | 321 (320–321) | 361 (358–362) | 2742 (2617–2793) | 3048 (2955–3093) | 13575 (13543–13735) | 364 (361–365) | 47.06 (45.68–47.53) |
| impr | 6 | 322 (321–322) | 357 (355–359) | 2745 (2687–2867) | 3072 (3011–3165) | 13503 (13375–13871) | 361.5 (360–364) | 47.23 (45.59–49.34) |
| impr-aa | 6 | 321.5 (321–323) | 358.5 (356–360) | 2728 (2641–2789) | 3041 (2967–3095) | 13527 (13487–13887) | 361.5 (360–366) | 47.21 (46.39–47.82) |

build: compile and link settings, IPC (ns, M msgs/s):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean | tput |
|---|---|---|---|---|---|---|---|---|
| impr | 5 | 294 (294–294) | 316 (316–320) | 2457 (2357–2509) | 9847 (9823–9855) | 14255 (14119–14439) | 346 (346–349) | 76.14 (69.92–76.68) |
| impr-c-x86-64 | 5 | 294 (294–294) | 317 (315–319) | 2395 (2305–2493) | 9823 (9799–9847) | 14279 (13863–14799) | 346 (344–349) | 76.23 (72.62–77.97) |
| impr-c-x86-64-v3 | 5 | 294 (293–295) | 316 (314–323) | 2393 (2135–2497) | 9823 (9767–9847) | 14287 (14071–14663) | 346 (341–348) | 75.49 (59.21–77.64) |
| impr-rust-x86-64 | 5 | 294 (294–294) | 316 (316–317) | 2435 (2153–2471) | 9839 (9823–9871) | 14231 (14087–14543) | 346 (344–348) | 73.38 (71.48–76.33) |
| impr-dynamic | 5 | 295 (294–297) | 317 (316–327) | 2329 (2325–2455) | 9847 (9807–9871) | 14615 (14583–15039) | 348 (346–358) | 73.55 (72.25–75.84) |

ipc-knobs: IPC settings (rtt in ns, max in µs, tput in M msgs/s):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean | max | tput |
|---|---|---|---|---|---|---|---|---|---|
| base | 5 | 294 (294–295) | 317 (315–321) | 2269 (2163–2515) | 9871 (9815–9879) | 14223 (14063–14647) | 346 (343–347) | 48 (35–81) | 76.58 (73.36–77.43) |
| nonsparse | 5 | 294 (294–295) | 317 (316–318) | 1676 (1666–1677) | 6723 (5519–9591) | 11791 (11583–11855) | 332 (331–333) | 34 (30–65) | 73.59 (72.71–77.43) |
| pretouch | 5 | 294 (294–294) | 319 (318–320) | 1269 (1205–1296) | 3287 (3001–4343) | 10479 (10423–10511) | 323 (322–323) | 56 (28–66) | 75.81 (73.78–79.46) |
| term1m | 5 | 296 (296–297) | 329 (328–330) | 1252 (1228–1298) | 9903 (9895–9911) | 11095 (11063–11167) | 344 (344–348) | 28 (22–49) | 90.41 (86.69–92.66) |
| shmhuge | 5 | 289 (289–289) | 314 (314–314) | 373 (370–375) | 2425 (1919–4311) | 10399 (10335–10551) | 303 (303–304) | 523 (487–614) | 77.99 (74.79–81.36) |
| combo | 5 | 290 (289–290) | 316 (315–319) | 371 (371–376) | 1808 (1736–2265) | 10143 (10135–10223) | 303 (301–304) | 43 (25–59) | 74.33 (73.23–75.91) |
| unpinned | 5 | 294 (294–295) | 319 (317–320) | 2387 (2337–2531) | 9863 (9807–9879) | 14503 (14215–14639) | 347 (339–349) | 33 (32–51) | 78.97 (71.86–94.00) |
| smt | 5 | 106 (106–106) | 121 (121–121) | 1897 (1891–1902) | 2091 (2073–2217) | 10191 (10015–10223) | 135 (135–136) | 48 (21–176) | 67.96 (67.60–68.30) |

ab: main against impr, loopback UDP rtt (µs):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean |
|---|---|---|---|---|---|---|---|
| main | 6 | 6.36 (6.34–6.40) | 6.52 (6.47–6.54) | 12.71 (12.62–12.78) | 21.57 (21.12–45.02) | 53.68 (48.96–61.82) | 6.39 (6.36–6.46) |
| impr | 6 | 6.39 (6.37–6.41) | 6.55 (6.52–6.58) | 12.71 (12.70–12.85) | 21.45 (20.35–22.86) | 52.08 (48.41–98.94) | 6.41 (6.36–6.61) |
| impr-aa | 6 | 6.37 (5.87–6.39) | 6.50 (6.47–6.55) | 12.68 (12.56–12.78) | 21.81 (20.21–24.02) | 56.16 (47.07–70.40) | 6.39 (6.35–6.44) |

UDP rerun (12 reps) as in ab, loopback UDP rtt (µs):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean |
|---|---|---|---|---|---|---|---|
| main | 12 | 6.40 (6.30–6.51) | 6.68 (6.62–6.79) | 13.13 (13.00–13.20) | 21.69 (20.53–65.60) | 60.77 (52.06–78.91) | 6.53 (6.46–6.74) |
| impr | 12 | 6.42 (6.32–6.53) | 6.68 (6.65–6.78) | 13.10 (12.97–13.23) | 22.09 (20.25–62.40) | 59.15 (50.49–79.81) | 6.50 (6.45–6.60) |
| impr-aa | 12 | 6.40 (6.34–6.50) | 6.68 (6.66–6.84) | 13.05 (12.97–13.33) | 22.02 (20.32–23.33) | 56.85 (52.00–70.59) | 6.50 (6.42–6.62) |

UDP rerun with pre-touched logs, loopback UDP rtt (µs):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean |
|---|---|---|---|---|---|---|---|
| main | 12 | 6.36 (6.29–6.42) | 6.60 (6.55–6.66) | 8.72 (8.57–8.88) | 20.37 (18.66–21.36) | 59.15 (53.79–73.15) | 6.29 (6.20–6.56) |
| impr | 12 | 6.37 (6.32–6.40) | 6.61 (6.58–6.63) | 8.76 (8.67–8.82) | 21.08 (18.72–22.25) | 55.98 (51.65–65.02) | 6.28 (6.22–6.33) |
| impr-aa | 12 | 6.37 (6.33–6.43) | 6.61 (6.59–6.66) | 8.71 (8.42–8.87) | 20.48 (19.04–22.78) | 55.86 (51.65–73.53) | 6.27 (6.21–6.37) |

udp-knobs: driver threading and idle strategy, loopback UDP rtt (µs):

| label | reps | p50 | p90 | p99 | p99.9 | p99.99 | mean |
|---|---|---|---|---|---|---|---|
| dedicated-noop | 5 | 6.36 (6.05–6.39) | 6.51 (6.49–6.56) | 12.76 (12.73–12.93) | 20.89 (19.90–21.25) | 50.59 (45.98–64.67) | 6.40 (6.36–6.70) |
| dedicated-spin | 5 | 6.39 (5.95–6.47) | 6.62 (6.60–6.65) | 12.84 (12.66–12.97) | 21.04 (20.22–42.81) | 58.14 (49.22–92.09) | 6.56 (6.38–6.64) |
| dedicated-backoff | 5 | 6.53 (5.90–42.59) | 42.75 (6.59–44.99) | 47.90 (12.84–50.81) | 52.29 (23.60–56.38) | 99.97 (67.84–131.20) | 14.22 (6.40–37.16) |
| shared-noop | 5 | 5.69 (5.66–5.75) | 6.32 (6.27–6.36) | 12.05 (11.98–12.34) | 15.26 (15.08–16.06) | 21.86 (21.49–46.66) | 6.09 (6.04–6.12) |
| shared-network-noop | 5 | 6.76 (5.53–6.81) | 6.86 (6.85–6.90) | 12.65 (12.55–12.93) | 15.01 (14.61–15.18) | 25.47 (22.72–47.23) | 6.82 (6.07–6.86) |
| dedicated-noop-smt | 5 | 6.41 (5.04–6.54) | 6.60 (6.53–6.67) | 12.70 (12.65–12.83) | 20.86 (20.05–21.74) | 51.23 (48.45–61.98) | 6.31 (5.83–6.55) |
| shared-network-noop-smt | 5 | 7.50 (7.47–7.53) | 7.60 (7.58–7.65) | 13.89 (13.86–13.97) | 16.53 (16.45–16.69) | 28.34 (23.25–40.58) | 7.70 (7.68–7.75) |

ps: idle poll cost, ns per poll (2 reps, unpinned), and time to LIVE:

| client | n | persistent subscription | plain subscription | time to LIVE (ms, each rep) |
|---|---|---|---|---|
| thread | 1 | 22.6 (22.4–22.8) | 10.1 (10.1–10.1) | 115, 112 |
| thread | 10 | 22.4 (22.2–22.5) | 8.3 (8.2–8.4) | 401, 562 |
| thread | 100 | 42.5 (42.4–42.6) | 10.1 (10.1–10.1) | 3622, 3622 |
| invoker | 1 | 59.0 (56.3–61.8) | 45.8 (45.7–45.9) | 29, 223 |
| invoker | 10 | 61.8 (61.4–62.2) | 11.7 (11.6–11.8) | 288, 294 |
| invoker | 100 | 106.8 (106.4–107.1) | 14.0 (13.4–14.6) | 827, 778 |

## Caveats

- **Scope.** Two runs on one day on 4-vCPU Azure VMs in two regions. Do not compare absolute numbers across the two hosts.
- **What was measured.** IPC and loopback UDP, one 32-byte message in flight, the driver and clients on one VM. Nothing here covers NICs, networks between hosts, loss or load, and the SHARED result may be specific to loopback.
- **Fresh logs.** Every run created new publications, so all results include first-touch effects unless a setting removes them: IPC runs wrote about 141 MB per direction (warm-up included) against 3 × 64 MiB of log, UDP runs about 22 MB against 3 × 16 MiB. The UDP rerun shows the size for UDP: with pre-touched logs, p99 fell 5% on AMD and from 13.1 to 8.8 µs on Intel.
- **Small boxes.** Four vCPUs leave two housekeeping CPUs for the driver and the client conductors; larger hosts may behave differently.
- **Not measured.** The time pre-touch adds when a publication or image is created, and huge pages on a dedicated tmpfs rather than `/dev/shm`.
- **ps.** The Java `ArchivingMediaDriver` from the test harness, unpinned, 2 reps, with the persistent loop timed before the plain one.
- **Mechanisms.** Explanations beyond what the tables show (TLB misses, cache residency, SMT effects) are hypotheses.
