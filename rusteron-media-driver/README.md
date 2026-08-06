# rusteron-media-driver

**rusteron-media-driver** is a Rust interface to the Aeron Media Driver, responsible for managing low-latency messaging infrastructure between producers and consumers. It's part of the [Rusteron](https://github.com/gsrxyz/rusteron) project and provides both standalone and embedded driver support.

> For production deployments, we recommend using the Aeron **Java** or **C** media driver.  
> The embedded version provided here is best suited for integration tests or lightweight environments.

---

## Sponsored by GSR

**Rusteron** is proudly sponsored and maintained by [GSR](https://www.gsr.io), a global leader in algorithmic trading and market making in digital assets.

It powers mission-critical infrastructure in GSR's real-time trading stack and is now developed under the official GSR GitHub organization as part of our commitment to open-source excellence and community collaboration.

We welcome contributions, feedback, and discussions. If you're interested in integrating or contributing, please open an issue or reach out directly.

---

## Installation

To use `rusteron-media-driver`, add the appropriate dependency to your `Cargo.toml`:

<details>
<summary>Dynamic</summary>

```toml
[dependencies]
rusteron-media-driver = "0.2"
```

</details>

<details>
<summary>Static</summary>

```toml
[dependencies]
rusteron-media-driver = { version = "0.2", features = ["static"] }
```

</details>

<details>
<summary>Static with precompiled C libs (macOS only)</summary>

```toml
[dependencies]
rusteron-media-driver = { version = "0.2", features = ["static", "precompile"] }
```

```toml
[dependencies]
rusteron-media-driver = { version = "0.2", features = ["static", "precompile-rustls"] }
```

</details>

Ensure the Aeron C libraries are properly installed and available on your system.

---

## Usage Examples

<details>
<summary>Standard Media Driver</summary>

```rust,no_run
// Launches a standalone Aeron Media Driver
use rusteron_media_driver::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let aeron_context = AeronDriverContext::new()?;
    aeron_context.set_dir(c"target/test")?;

    let aeron_driver = AeronDriver::new(&aeron_context)?;
    aeron_driver.start(false)?;
    println!("Aeron Media Driver started");

    Ok(())
}
```

</details>

<details>
<summary>Embedded Media Driver</summary>

```rust
// Embeds the media driver directly into the current process
use rusteron_media_driver::*;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let media_driver_ctx = AeronDriverContext::new()?;
    let (stop, driver_handle) = AeronDriver::launch_embedded(media_driver_ctx.clone(), false);

    let ctx = AeronContext::new()?;
    ctx.set_dir(&cformat!("{}", media_driver_ctx.get_dir()))?;

    thread::sleep(Duration::from_secs(3)); // Simulated workload

    stop.store(true, Ordering::SeqCst);
    driver_handle.join().expect("Failed to join driver thread");
    println!("Embedded Aeron Media Driver stopped");

    Ok(())
}
```

</details>

---

## DPDK ENA transport (optional, Linux x86_64)

The `dpdk` feature swaps the kernel-socket sender/receiver for a DPDK ENA
kernel-bypass transport on Amazon Linux 2023 / EKS Nitro. It is selected with
one environment variable, identically in the standalone binary and the typed
embedded API:

```sh
RUSTERON_MEDIA_DRIVER_TRANSPORT=dpdk-ena cargo run -p rusteron-media-driver --features dpdk --bin media_driver
```

- Absent or `default` (the default) → unchanged socket behaviour.
- `dpdk-ena` → the DPDK transport. Any configuration, validation, or native
  failure exits nonzero and never falls back to the socket driver.
- Anything else → invalid, exits nonzero.

The DPDK configuration is read from the environment (see
`rusteron-media-driver/src/dpdk/env.rs`): the required values are
`RUSTERON_DPDK_FILE_PREFIX`, `RUSTERON_DPDK_SENDER_PCI`,
`RUSTERON_DPDK_SENDER_IPV4_CIDR`, `RUSTERON_DPDK_SENDER_GATEWAY`, and the
`RUSTERON_DPDK_RECEIVER_*` twins. The Aeron context must be a dedicated-thread
driver with distinct sender/receiver CPUs, spin idle strategies, disjoint
wildcard port ranges, and an MTU no larger than `RUSTERON_DPDK_MAX_AERON_MTU`;
these are set through the standard `AERON_*` environment variables
(`AERON_SENDER_CPU_AFFINITY`, `AERON_SENDER_IDLE_STRATEGY`,
`AERON_SENDER_WILDCARD_PORT_RANGE`, `AERON_MTU_LENGTH`, …).

For an embedded deployment that selects the transport through the typed Rust
API instead of the environment, see
[`examples/embedded_dpdk.rs`](examples/embedded_dpdk.rs).

---

## Contributing & License

See the root [README](https://github.com/gsrxyz/rusteron#readme) and [CONTRIBUTING.md](https://github.com/gsrxyz/rusteron/blob/main/CONTRIBUTING.md). Build requirements are in [BUILD.md](https://github.com/gsrxyz/rusteron/blob/main/BUILD.md).
Dual-licensed under MIT or Apache-2.0.

---

## Acknowledgments

Special thanks to:

* [@mimran1980](https://github.com/mimran1980), a core low-latency developer at GSR and the original creator of Rusteron - your work made this possible!
* [@bspeice](https://github.com/bspeice) for the original [`libaeron-sys`](https://github.com/bspeice/libaeron-sys)
* The [Aeron](https://github.com/real-logic/aeron) community for open protocol excellence
