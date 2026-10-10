# Rusteron

[![Crates.io](https://img.shields.io/crates/v/rusteron-archive)](https://crates.io/crates/rusteron-archive)
[![CI](https://github.com/gsrxyz/rusteron/actions/workflows/ci.yml/badge.svg)](https://github.com/gsrxyz/rusteron/actions/workflows/ci.yml)
[![API Docs](https://docs.rs/rusteron-archive/badge.svg)](https://docs.rs/rusteron-archive/)
[![github API Docs](https://custom-icon-badges.demolab.com/badge/githubdocs-blue.svg?logo=log\&logoSource=feather)](https://gsrxyz.github.io/rusteron)

> **Rusteron** is a thin, high-performance Rust wrapper over the [Aeron](https://github.com/real-logic/aeron) C API.
> It exposes low-level C bindings with minimal abstraction, optimized for production use in latency-sensitive environments.

---

## Sponsored by GSR

**Rusteron** is proudly sponsored and maintained by [GSR](https://www.gsr.io), a global leader in algorithmic trading and market making in digital assets.

It powers mission-critical infrastructure in GSR's real-time trading stack and is now developed under the official GSR GitHub organization as part of our commitment to open-source excellence and community collaboration.

We welcome contributions, feedback, and discussions. If you're interested in integrating or contributing, please open an issue or reach out directly.

---

## Project Overview

This project builds on a fork of [`libaeron-sys`](https://github.com/bspeice/libaeron-sys), offering Rust access to Aeron’s native C API. The API is **not fully idiomatic**, but is auto-generated for consistency and reliability. This tradeoff supports:

* Performance-sensitive trading environments
* Minimal runtime overhead
* Low maintenance costs

**Warning**: This library operates in an `unsafe` context and requires care. Improper usage (e.g., using a publisher after the Aeron client is closed) can lead to **undefined behavior or segmentation faults**.

---

## Module Overview

| Module                                                                                            | Description                                                                |
| ------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| [`rusteron-code-gen`](https://github.com/gsrxyz/rusteron/tree/main/rusteron-code-gen)             | Code generation engine to produce consistent Rust bindings for Aeron C.    |
| [`rusteron-client`](https://github.com/gsrxyz/rusteron/tree/main/rusteron-client)                 | Core Aeron client wrapper (connect, publish, subscribe).                   |
| [`rusteron-archive`](https://github.com/gsrxyz/rusteron/tree/main/rusteron-archive)               | Adds stream recording and replay features. Includes `rusteron-client`.     |
| [`rusteron-media-driver`](https://github.com/gsrxyz/rusteron/tree/main/rusteron-media-driver)     | Rust interface for launching an embedded or standalone Aeron Media Driver. |
| [`rusteron-docker-samples`](https://github.com/gsrxyz/rusteron/tree/main/rusteron-docker-samples) | Sample Docker setups for media driver + pub/sub flows. Not prod-ready.     |

Note: `rusteron-archive` includes `rusteron-client`, so you do **not** need both as dependencies.

---

## Installation

Choose the module and linking style appropriate for your project.

**Dynamic library:**

```toml
[dependencies]
rusteron-client = "0.2"
```

**Static library:**

```toml
[dependencies]
rusteron-client = { version = "0.2", features = ["static"] }
```

**Precompiled static libs (macOS and Linux, no cmake or Java needed):**

```toml
[dependencies]
rusteron-client = { version = "0.2", features = ["static", "precompile"] }
```

**Precompiled static libs with rustls downloader (macOS and Linux):**

```toml
[dependencies]
rusteron-client = { version = "0.2", features = ["static", "precompile-rustls"] }
```

Replace `rusteron-client` with `rusteron-archive` or `rusteron-media-driver` as needed.

For full build instructions, see [BUILD.md](./BUILD.md).

### CPU target

A release build from source compiles the Aeron C code for the build machine's CPU
(`-march=native`). Set `RUSTERON_C_MARCH` (e.g. `x86-64-v3`) when binaries run on
machines other than the one that built them. The precompiled libraries (`precompile`)
target the architecture's baseline (`x86-64`, `armv8-a`), so they run on any CPU. On the
x86-64 VMs in [Tuning on x86-64 Linux](#tuning-on-x86-64-linux), `-march=native` for the
client's C code showed no consistent gain over `x86-64` or `x86-64-v3` on IPC; the driver's
build and UDP were not varied.

### Multi-threaded (`Sync`) handles

Publication, subscription, counter and counters-reader handles are `Send` but **not `Sync`**
by default; they use `Rc` and may be moved to one owning thread. The `Aeron` client and
`AeronExclusivePublication` become `Send` only under `multi-threaded`; other handles
(contexts, images) stay on the thread that created them. Enable `multi-threaded` to swap
`Rc` → `Arc` and add `unsafe impl Sync`, so `&Handle` can be shared across threads for the
ops Aeron C documents as thread-safe
(`offer` / `try_claim` / `position` / `is_connected`):

```toml
[dependencies]
rusteron-client = { version = "0.2", features = ["multi-threaded"] }
```

```rust,ignore
// AeronPublication is Sync under `multi-threaded`, so threads share `&publication`.
std::thread::scope(|s| -> Result<(), AeronOfferError> {
    let a = s.spawn(|| publication.offer(b"hello"));
    let b = s.spawn(|| publication.offer(b"world"));
    a.join().expect("publisher thread panicked")?;
    b.join().expect("publisher thread panicked")?;
    Ok(())
})?;
```

> **The flag only lifts the Rust-side barrier — it does not make the underlying Aeron
> object thread-safe.** Sharing is correct only for objects Aeron C documents as
> thread-safe for concurrent use, e.g. `AeronPublication` (`ConcurrentPublication`),
> `AeronCounter`, and `Aeron` itself. `AeronSubscription` is documented by Aeron as
> **not** threadsafe and must not be shared between subscribers — it stays `Send`-only
> (never `Sync`) even under `multi-threaded`, so it can be moved to one other thread but
> never accessed concurrently from several. `AeronExclusivePublication` is
> single-producer by design and must not be shared across threads either (it only gains
> `Send` under `multi-threaded`, never `Sync`). It is the caller's responsibility to
> check the thread-safety of each object before sharing `&Handle`.

---

## Documentation & Guides

For detailed guides and code snippets on Aeron features in Rust, see:
- [Multi-Destination Subscription (MDC / MDS) Guide](./docs/mdc_mds_guide.md)
- [Media Driver Configuration & Back-pressure Guide](./docs/media_driver_configuration_and_backpressure_guide.md)

---

## Development

Build tasks use [`just`](https://github.com/casey/just). Run `just` to list commands, or `cargo install just` if needed.

---

## Example: Pub/Sub

<details>
<summary>Expand for usage example</summary>

```rust,no_run
use rusteron_client::{
    cformat, Aeron, AeronContext, AeronErrorHandlerLogger, AeronHeader, BackoffIdleStrategy,
    Handlers, IdleStrategy,
};
use rusteron_media_driver::testing::EmbeddedDriver;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Unique directory; stopped and joined when `driver` drops, after the client.
    let driver = EmbeddedDriver::launch()?;

    let ctx = AeronContext::new()?;
    ctx.set_dir(&cformat!("{}", driver.dir()))?;
    // Error handler is Option (None = silently drop async client errors).
    // The Aeron samples always set a logger so failures are visible.
    ctx.set_error_handler(Some(AeronErrorHandlerLogger))?;
    let aeron = Aeron::new(&ctx)?;
    aeron.start()?;

    // c"..." literals are compile-time &'static CStr — zero runtime cost.
    let channel = c"aeron:ipc";
    let publication = aeron
        .async_add_publication(channel, 123)?
        .poll_blocking(Duration::from_secs(5))?;
    let subscription = aeron
        .async_add_subscription(
            channel, 123,
            Handlers::NONE,
            Handlers::NONE,
        )?
        .poll_blocking(Duration::from_secs(5))?;

    // offer returns Ok(position) or a typed AeronOfferError; retry the
    // retryable ones (back-pressure / admin action / not connected),
    // surface the fatal ones (closed / max position exceeded).
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut idle = BackoffIdleStrategy::new();
    loop {
        match publication.offer(b"Hello, Aeron!") {
            Ok(_) => break,
            Err(e) if e.is_retryable() && Instant::now() < deadline => idle.idle(0),
            Err(e) => return Err(e.into()),
        }
    }

    // The subscription's image can appear just after the offer succeeds, so poll
    // until the message arrives. `poll_fn` runs the closure per fragment (zero
    // allocation); use a fragment assembler for messages larger than the MTU.
    let mut received = false;
    while !received {
        if Instant::now() >= deadline {
            return Err("no message received".into());
        }
        let fragments = subscription.poll_fn(
            |msg: &[u8], header: AeronHeader| {
                println!("received {} bytes at position {}", msg.len(), header.position());
                received = true;
            },
            10,
        )?;
        idle.idle(fragments);
    }
    Ok(())
}
```

> **`poll_blocking` / `add_*(.., timeout)` are for example brevity only** — they block the
> calling thread in a busy-poll loop. In production, drive the async poller's `poll()` from
> your own event loop. See
> [`rusteron-client/examples/non_blocking_publisher.rs`](rusteron-client/examples/non_blocking_publisher.rs)
> for the idiomatic pattern.

</details>

### C strings without hidden allocations

Every channel/URI argument is a `&CStr` (the C API's type), so every heap allocation is
visible at the call site. Recommended three-tier pattern, cheapest first:

```rust,ignore
// 1. Constant channels: c"..." literals — compile-time &'static CStr, zero runtime cost.
aeron.async_add_publication(c"aeron:ipc", 10)?;

// 2. Dynamic URIs: cformat! — ONE named heap allocation (format + CString in one step).
let uri = cformat!("aeron:udp?endpoint=localhost:{port}");
aeron.async_add_publication(&uri, 10)?;

// 3. Repeated paths: build the CString once, store it, pass `&it` (zero-copy on reuse).
let chan: std::ffi::CString = cformat!("aeron:udp?endpoint={endpoint}");
for _ in 0..reconnect_attempts {
    aeron.async_add_publication(&chan, 10)?; // no allocation per call
}
```

---

## What's new in 0.2

0.2 is a breaking release (deeper write-up in the [rusteron-client README](./rusteron-client/README.md#general-patterns)):

- **Deferred close.** `aeron.close()` no longer frees child resources immediately (the 0.1.x behaviour was a use-after-free). Close is deferred until the last reference drops; drop order is arbitrary. `unsafe close_now()` forces immediate teardown.
- **Reference-counted handlers.** `Handler::leak()`/`release()` are gone; `Handler::new()` is `Arc`-backed and freed when the last clone drops. Retained-callback setters take the value (closure or trait impl) and return the `Handler`. `Handlers::NONE` covers "no callback".
- **Typed errors.** `offer`/`try_claim` return `Result<i64, AeronOfferError>` with `is_retryable()`. `AeronCError` construction is allocation-free (never reads `aeron_errmsg()`); `capture_errmsg()` opts into attaching the message. Archive control ops return `Result<_, AeronArchiveError>` (`From` keeps `?` working).
- **Hot path.** `offer_parts(&[&header, &payload])` publishes several buffers as one message with no intermediate Vec; C-string args follow the `c""`/`cformat!`/reuse pattern above.
- **Convenience.** `Aeron::connect_dir`, `AeronDriver::launch_embedded_guard` (RAII), `ChannelUri::add_session_id`, `AeronUriStringBuilder::ipc()/udp()`, retained-image accessors, direct constant getters, and ported samples (basic_publisher/subscriber, ping/pong, file transfer, MDS, request/response).

## Migrating from 0.1.168 to 0.2

Old → new for every renamed/changed API

| 0.1.168 | 0.2 | Notes |
|---|---|---|
| `Handler::leak(h)` + manual `handler.release()` | `Handler::new(h)` | Freed automatically when the last clone drops; registering resources keep clones. |
| `ctx.set_error_handler(Some(&handler))` (borrowed) | `ctx.set_error_handler(Some(handler_or_closure))` | Retained setters take the value; closures work directly; returns the `Handler`. |
| `aeron.close()` — freed children immediately | deferred close | Frees when the last reference drops; `unsafe close_now()` forces immediate. |
| `Handler` was `Sync` | `Send` only | The conductor thread invokes callbacks; sharing `&Handler` across threads raced. |
| `AeronPublication` / `AeronSubscription` / … were `Sync` | `Send` only by default; `Sync` only under `multi-threaded`, and only for types Aeron C documents as thread-safe | Handles use `Rc` (single-thread ownership). Enable the `multi-threaded` feature (`Rc` → `Arc`) to get `unsafe impl Sync` for `AeronPublication`/`AeronCounter`/`Aeron` (share `&Handle` across threads for `offer` / `try_claim` / `position` / `is_connected`). `AeronSubscription` and `AeronExclusivePublication` are documented by Aeron as not safe to share and remain `Send`-only (never `Sync`), even with `multi-threaded`. |
| `publication.offer(buf, supplier)` → raw `i64` | `publication.offer_raw(buf, supplier)` | Same branch-free sentinel return, renamed to make "raw" explicit. |
| `publication.offer_result(buf, supplier)` → `Result<_, AeronCError>` | `publication.offer_with_reserved_value(buf, supplier)` → `Result<_, AeronOfferError>` | Typed offer errors with `is_retryable()`. |
| `publication.offer_result_simple(buf)` | `publication.offer(buf)` | The common no-supplier case is now the flagship name. |
| `try_claim_result(len, claim)` / `try_claim_owned` → `AeronCError` | `try_claim(len, claim)` / `try_claim_owned` → `AeronOfferError` | Same RAII `AeronClaim`; typed error. |
| `subscription.poll_once(f, limit)` | `subscription.poll_fn(f, limit)` | Renamed (`_once` read as "one fragment"); same on `AeronImage` / `AeronArchiveReplayMerge`. |
| `subscription.for_each_fragment(limit, f)` | `subscription.poll_fn(f, limit)` | Removed (alias with the arguments in the opposite order). |
| `sub.poll(assembler.process(&mut ctx, f), limit)` | `assembler.poll(&sub, &mut ctx, f, limit)` | `process()` leaked a raw ctx pointer past the borrow (UAF hazard); the new form scopes it. |
| `Handlers::no_available_image_handler()`, `no_unavailable_image_handler()`, … | `Handlers::NONE` | One constant, any callback parameter, full inference. Old helpers removed. |
| `pub.add_destination(&aeron, dest, timeout)` (`&mut self`) | `pub.add_destination(dest, timeout)` (`&self`) | Owning `Aeron` comes from the handle's dependency graph. Same for subscriptions / exclusive publications. |
| `AeronCnc::new(dir)` | `AeronCnc::open(&CStr)` or `AeronCnc::read(&CStr, \|cnc\| { … })` | `new_on_heap`/`read_on_partial_stack` renamed; now accept `&CStr` (not `&str`/`&CString`). `read` = scoped (zero-alloc, preferred for one-shot), `open` = owned handle (for repeated polling). |
| `&"aeron:ipc".into_c_string()` (allocates at runtime) | `c"aeron:ipc"` | See "C strings without hidden allocations" above; `cformat!` for dynamic URIs. |
| `wrapper.get_inner_mut()` / `ManagedCResource::get_mut()` (safe) | `unsafe …()` | `&mut` from `&self`; the caller must now promise exclusive access. The only internal caller (`clone_struct`) is wrapped in `unsafe` already. |
| `AeronUriStringBuilder::put_string(&CStr, &str)` / `put_strings(&str, &str)` | `AeronUriStringBuilder::put_str(&CStr, &str)` | Single name, single key type (`&CStr` — pair with `c"media"` or `CStr::from_bytes_until_nul`); `put_strings` removed (no external callers). |
| `archive.start_replay(...)`, `start_recording(...)`, … → `Result<_, AeronCError>` | `Result<_, AeronArchiveError>` | Control ops on `AeronArchive` return the typed error (parseable code + message). Constructors, async-connect, and context setters still return `AeronCError`; `From<AeronArchiveError> for AeronCError` keeps `?` working across the boundary. |
| `async_add_exclusive_publication.poll(...).get_registration_id()` on the deprecated `exclusive_exclusive` alias | only `aeron_async_add_exclusive_publication_get_registration_id` is exposed | The deprecated `aeron_async_add_exclusive_exclusive_publication_get_registration_id` C alias is dropped (it collided with the canonical name); use the canonical `get_registration_id()`. |
| `DarwinPthread*`, `OpaquePthread*` wrapper structs in the generated API | removed | Bindgen pthread internals are no longer emitted as wrapper types; socket types the driver wrappers reference (`sockaddr_storage`, `iovec`, …) are retained. |

Behavioural notes:
- A panicking fragment handler aborts the process (panic cannot unwind across the C
  callback boundary) — return instead of panicking in handlers.
- `AeronCError` construction never reads or copies `aeron_errmsg()` — retry loops
  (e.g. polling that keeps returning `-1`) stay allocation-free. `Display` /
  `get_last_err_message()` read the live buffer; call `err.capture_errmsg()` at the
  error site to pin the text to the error when you store it or log it later.

For recording, replay, and **persistent subscriptions** (replay history, then seamlessly join a
live stream), see [`rusteron-archive`](./rusteron-archive/README.md#persistent-subscriptions).

---

## Tuning on x86-64 Linux

Measured on 2026-10-09 between two Azure `Standard_D8s_v6` VMs (Intel Xeon Platinum 8573C) in one proximity placement group, on Debian 13 with Linux 7.2.6: UDP round trips and throughput between the two hosts, and IPC on each host, with 32-byte messages. Each figure is a median of 5 interleaved runs. Other NICs and clouds will differ. Tables, method and open issues are in [BENCHMARKS.md](./BENCHMARKS.md#intel-d8s_v6-pair-on-azure-2026-10-09).

These were used in every run, and earlier runs on smaller VMs showed each to help:
- `AERON_TERM_BUFFER_SPARSE_FILE=false` (driver) and `AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true` (clients). Without them a new publication page-faults its way through its log.
- `AERON_DIR` on 2 MiB `hugetlbfs` with `AERON_FILE_PAGE_SIZE=2097152`.
- `noop` idle for the driver's sender and receiver.
- Every busy-spinning thread pinned to its own core, and everything else (interrupts, the driver's conductor, the clients' other threads) on a housekeeping CPU.

| Setting | Measured effect | Cost |
|---|---|---|
| Socket buffers and receiver window: `AERON_SOCKET_SO_SNDBUF`, `AERON_SOCKET_SO_RCVBUF` and `AERON_RCV_INITIAL_WINDOW_LENGTH` at 2 MiB, with `net.core.rmem_max`/`wmem_max` raised to allow them | UDP throughput between the hosts 1.9 → 13 M msgs/s. No effect on round-trip latency with one message in flight. | Memory per socket. |
| MTU 9000 inside the VNet and `AERON_MTU_LENGTH=8192` | UDP throughput 13 → 28.5 M msgs/s. | Azure allows it only inside a VNet and directly peered VNets; keep the default route at 1500 so traffic leaving the VNet still fits. |
| CPU isolation: `isolcpus=nohz,domain,managed_irq,<hot CPUs> nohz_full=<same> rcu_nocbs=<same> irqaffinity=<housekeeping>` | IPC p99.99 2.5 → 0.8–1.0 µs, and most runs' max 20–67 → 4–7 µs. UDP: no change on its own. | Isolated CPUs take no unpinned work, which then all crowds onto the housekeeping CPUs, so every busy thread must be pinned. |
| Socket busy reads, `net.core.busy_read=50`, with `napi_defer_hard_irqs=2` and `gro_flush_timeout=200000` on the NIC's VF, on isolated CPUs | UDP round trip p50 38.6 → 31 µs, p99.99 157 → 44–50 µs. `net.core.busy_poll` does nothing for Aeron's receiver, which calls `recvmmsg` directly. | The receiver's core polls the NIC queue itself. |
| The NIC's queue IRQs on the receiver's core, on isolated CPUs | UDP p99.99 157 → 49–57 µs; with the tuned kernel below, max 74 µs (62–84). | Re-pin them whenever Azure re-adds the VF (host servicing), which spreads them over all CPUs again. |
| The client's other threads (its conductor) off the busy-spinning CPUs | IPC p99.99 3.1–3.6 → 0.7–1.0 µs on isolated CPUs. | A CPU for them, or the conductor agent invoker. |
| A tuned kernel on top of isolation: `nosmt idle=poll rcu_nocb_poll nowatchdog nmi_watchdog=0 nosoftlockup skew_tick=1 transparent_hugepage=never audit=0`, all IRQs and workqueues on the housekeeping CPU, background services stopped | UDP p50 38.6 → 34.9 µs. IPC unchanged. | SMT off halves the CPUs; `idle=poll` keeps every idle CPU busy. |
| `mitigations=off` | UDP p50 34.9 → 33.9 µs and p99.99 156 → 121 µs. IPC unchanged. | Turns off the kernel's protection against CPU side-channel attacks. |
| Driver threading | Dedicated (pinned `noop` sender and receiver), SHARED_NETWORK and SHARED were within about 2 µs of each other at UDP p50. A SHARED driver costs IPC: p50 0.31 → 0.36–0.42 µs and half the throughput. | Dedicated spins two cores. |
| Across regions, socket buffers and initial window sized to the round trip: 16 MiB for 53 ms between West US 3 and North Central US, over 64 MiB terms, with `net.core.rmem_max`/`wmem_max` raised to match and kept in `/etc/sysctl.d` | Throughput 18 → 119 MB/s on a clean link. At 0.1% loss both carried 10–13 MB/s. | Memory per socket. If `rmem_max` resets (a reboot undoes `sysctl -w`), the driver refuses to start, as the window exceeds the receive buffer. |
| In Kubernetes: kubelet's static CPU manager and Guaranteed pods requesting whole CPUs | Each container gets its CPUs to itself, with no CFS quota. Under the default policy, containers limited to exactly their spinning threads' CPUs were throttled (23–55% of 100 ms periods for a 1-CPU driver); one CPU of headroom or no CPU limit avoided it. | See [BENCHMARKS.md](./BENCHMARKS.md#recommended-settings-for-kubernetes) for the kubelet and pod settings. |

Packet loss costs far more than its share:
- 0.1% UDP loss cut same-zone throughput by a third, and 1% by 91%. Every lost packet was still retransmitted.
- In request/response traffic, a lost message waits for the sender's next heartbeat, 100 ms in the C driver.
- Find and fix the loss before tuning anything else ([details](./BENCHMARKS.md#packet-loss-and-distance-west-us-3-and-north-central-us-2026-10-10)).

```bash
# C media driver (a Java driver reads -D system properties instead)
export AERON_TERM_BUFFER_SPARSE_FILE=false
export AERON_SENDER_IDLE_STRATEGY=noop AERON_RECEIVER_IDLE_STRATEGY=noop
export AERON_CONDUCTOR_CPU_AFFINITY=0 AERON_SENDER_CPU_AFFINITY=4 AERON_RECEIVER_CPU_AFFINITY=6
export AERON_SOCKET_SO_SNDBUF=2m AERON_SOCKET_SO_RCVBUF=2m AERON_RCV_INITIAL_WINDOW_LENGTH=2m
export AERON_FILE_PAGE_SIZE=2097152 AERON_DIR=/mnt/huge/aeron
# every client
export AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true
# the host
sudo sysctl -w net.core.rmem_max=16777216 net.core.wmem_max=16777216 net.core.busy_read=50
# the NIC's VF (its netdev has eth0 as master), as measured with busy_read
for i in /sys/class/net/*; do [ -e "$i/master" ] && vf=$(basename "$i"); done
echo 2 | sudo tee /sys/class/net/$vf/napi_defer_hard_irqs
echo 200000 | sudo tee /sys/class/net/$vf/gro_flush_timeout
sudo sysctl -w vm.nr_hugepages=1536
sudo mkdir -p /mnt/huge && sudo mount -t hugetlbfs -o pagesize=2M,size=2G,uid="$(id -u)" none /mnt/huge
```

### Archive disks

Measured on 2026-10-10 with a Java Archive on two Azure `Standard_D8ds_v6`, recording 1 KiB messages from up to 16 streams ([details](./BENCHMARKS.md#java-archive-on-disk-under-load-two-d8ds_v6-2026-10-10)).

- **Size the disk for sustained throughput, not IOPS.**
  - At file sync level 0 (the default) the archive writes into the page cache. RAM takes the bursts, so the disk only needs the average write rate plus replay reads.
  - Cloud disks share one budget between reads and writes.
  - Recordings are sequential, so the 3,000 IOPS a disk includes is plenty.
- **A durable network disk is enough.** On Azure that is Premium SSD v2: 125 MB/s included, more bought separately. AWS gp3 is priced the same way but wasn't measured here.
  - The VM caps its disks as well: 424 MB/s in total on a D8ds_v6.
  - New Premium SSD v2 disks, freshly formatted with a full-device discard (plain `mkfs.xfs`), ran well below their provisioned rate for their first 25 minutes or more. Test a fresh disk before relying on it.
- **Let the page cache hold larger bursts:** `sudo sysctl -w vm.dirty_bytes=17179869184 vm.dirty_background_bytes=536870912`. That allows 16 GiB of unwritten data, half of the 32 GiB VM, so scale it to your RAM.
  - With these, a 125 MB/s disk took 30 s bursts of 400 MB/s at full rate; with Linux's defaults it throttled after 15 s.
  - Data still in memory is lost if the host fails, which level 0 already accepts.
  - A full 16 GiB takes over 2 minutes to write out at 125 MB/s, at shutdown too.
- **Leave headroom for replays.** While recording ran flat out at the disk's limit, replays from that disk slowed from about 1 GB/s to 67–246 MB/s on local NVMe.
- **Local NVMe was the fastest and costs nothing extra:** 545 MB/s sustained, 1 GB/s of replays, and `fdatasync` in 0.03 ms against 0.8 ms on Premium SSD v2. Its data is gone when the VM stops or its host fails, though, so use it only for an archive that is replicated elsewhere.
- **File sync level 1 or 2** makes every write wait for the disk. That cost nothing sustained on local NVMe. On Premium SSD v2 it was measured only while the new disks were still slow, at 56–220 MB/s, so its cost at full speed is unknown.

---

## Benchmarks

For latency and throughput benchmarks, refer to [BENCHMARKS.md](./BENCHMARKS.md).

---

## Contributing

Contributions are more than welcome! Please:

* Submit bug reports, ideas, or improvements via GitHub Issues
* Propose changes via pull requests
* Read our [CONTRIBUTING.md](https://github.com/gsrxyz/rusteron/blob/main/CONTRIBUTING.md)

We’re especially looking for help with:

* API design reviews
* Safety and idiomatic improvements
* Dockerized and deployment examples

---

## License

Licensed under either [MIT License](https://opensource.org/licenses/MIT) or [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0) at your option.

---

## Acknowledgments

Special thanks to:

* [@mimran1980](https://github.com/mimran1980), a core low-latency developer at GSR and the original creator of Rusteron - your work made this possible!
* [@bspeice](https://github.com/bspeice) for the original [`libaeron-sys`](https://github.com/bspeice/libaeron-sys)
* The [Aeron](https://github.com/real-logic/aeron) community for open protocol excellence
