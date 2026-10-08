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
<summary>Static with precompiled C libs (macOS / Linux)</summary>

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
// A standalone media driver; `cargo run -p rusteron-media-driver --bin media_driver` ships the same.
use rusteron_media_driver::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = AeronDriverContext::new()?;
    context.set_dir(c"target/aeron")?;
    let driver = AeronDriver::new(&context)?;
    driver.start(true)?; // run the conductor duty cycle on this thread
    println!("media driver running in {}", context.get_dir());
    loop {
        driver.main_idle_strategy(driver.main_do_work()?);
    }
}
```

</details>

<details>
<summary>Embedded Media Driver</summary>

```rust,no_run
// Embeds the media driver in this process: unique directory, stopped and joined on drop.
use rusteron_media_driver::testing::EmbeddedDriver;
use rusteron_media_driver::Aeron;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let driver = EmbeddedDriver::launch()?; // or EmbeddedDriver::launch_with(|ctx| { /* tune ctx */ Ok(()) })
    // Applications usually connect with rusteron_client::Aeron::connect_dir(driver.dir()).
    let aeron = Aeron::connect_dir(driver.dir())?;
    // add publications / subscriptions on `aeron` ...
    drop(aeron); // close the client before the driver stops
    Ok(())
}
```

For a caller-built `AeronDriverContext`, `AeronDriver::launch_embedded_guard(ctx, false)` is the RAII form (stops and joins on drop).

</details>

---

## Building C against Aeron's headers

The crate declares `links = "aeron_driver"`, so a dependent's build script can compile C
(a custom UDP transport, say) against the vendored Aeron sources:

| Variable | Directory |
|---|---|
| `DEP_AERON_DRIVER_INCLUDE` | the media driver's headers |
| `DEP_AERON_DRIVER_CLIENT_INCLUDE` | the client headers they include |
| `DEP_AERON_DRIVER_AERON_ROOT` | the Aeron source tree |

## Contributing & License

See the root [README](https://github.com/gsrxyz/rusteron#readme) and [CONTRIBUTING.md](https://github.com/gsrxyz/rusteron/blob/main/CONTRIBUTING.md). Build requirements are in [BUILD.md](https://github.com/gsrxyz/rusteron/blob/main/BUILD.md).
Dual-licensed under MIT or Apache-2.0.

---

## Acknowledgments

Special thanks to:

* [@mimran1980](https://github.com/mimran1980), a core low-latency developer at GSR and the original creator of Rusteron - your work made this possible!
* [@bspeice](https://github.com/bspeice) for the original [`libaeron-sys`](https://github.com/bspeice/libaeron-sys)
* The [Aeron](https://github.com/real-logic/aeron) community for open protocol excellence
