# rusteron-media-driver

**rusteron-media-driver** is a module in the **rusteron** project that provides an interface to the Aeron Media Driver in a Rust environment. This module is crucial for managing the messaging infrastructure between producers and consumers, allowing for efficient low-latency communication.

## Overview

The **rusteron-media-driver** module is designed to help Rust developers interact with the Aeron Media Driver, which is responsible for managing the communication between different Aeron clients. This module provides a wrapper around the Aeron C Media Driver API and offers both standard and embedded driver options for use in different environments.

The media driver can be used to set up the messaging directory and manage data streams. The embedded media driver is particularly useful for testing purposes or for simplifying deployment scenarios where a separate media driver process is not needed.

## 🚀 Sponsorship & Adoption

🏢 Sponsored by GSR

This project is proudly sponsored and maintained by *GSR*, a global leader in algorithmic trading and market making in digital assets.
Rusteron plays a foundational role in GSR’s trading platform technology stack, providing critical infrastructure for performance-sensitive, real-time systems. As part of our commitment to engineering excellence and open collaboration, the project is now developed and maintained under GSR’s official GitHub organization.
We believe in sharing robust, production-tested tools with the broader community and welcome external contributions, feedback, and discussion.
If you're interested in contributing or partnering with us, feel free to reach out or open an issue!

## Usage Note

It is recommended to run the media driver using the Aeron Java or C version for production use. This crate can also be used to start the media driver embedded within unit or integration tests.

## Installation

To use **rusteron-media-driver**, add it to your `Cargo.toml`:

dynamic lib
```toml
[dependencies]
rusteron-media-driver = "0.1"
```

static lib
```toml
[dependencies]
rusteron-media-driver = { version = "0.1", features= ["static"] }
```

static lib with precompiled c libs (mac os x only)
```toml
[dependencies]
rusteron-media-driver = { version = "0.1", features= ["static", "precompile"] }
```

Ensure that you have also set up the necessary Aeron C libraries required by **rusteron-media-driver**.

## Features

- **Media Driver Management**: Start, stop, and configure an Aeron Media Driver instance.
- **Embedded Media Driver**: Launch an embedded media driver directly from your code for testing purposes.

## Usage Examples

### Standard Media Driver Example

```rust,no_ignore
use rusteron_media_driver::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let aeron_context = AeronDriverContext::new()?;
    aeron_context.set_dir(&"target/test".into_c_string())?;

    // Create Aeron driver
    let aeron_driver = AeronDriver::new(&aeron_context)?;
    aeron_driver.start(false)?;
    println!("Aeron Media Driver started");
    
    Ok(())
}
```

### Embedded Media Driver Example

```rust,no_ignore
use rusteron_media_driver::*;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and launch an embedded media driver
    let media_driver_ctx = AeronDriverContext::new()?;
    let (stop, driver_handle) = AeronDriver::launch_embedded(media_driver_ctx.clone(), false);

    // Create Aeron context and link with the embedded driver
    let ctx = AeronContext::new()?;
    ctx.set_dir(&media_driver_ctx.get_dir().into_c_string())?;
    
    // Wait a bit for demonstration purposes
    thread::sleep(Duration::from_secs(3));

    // Stop the driver
    stop.store(true, Ordering::SeqCst);
    driver_handle.join().expect("Failed to join driver thread");
    println!("Embedded Aeron Media Driver stopped");

    Ok(())
}
```



## Building This Project Instructions

For detailed instructions on how to build **rusteron**, please refer to the [HOW_TO_BUILD.md](../HOW_TO_BUILD.md) file.

## Contributing

Contributions are more than welcome! Please see our [contributing guidelines](https://github.com/gsrxyz/rusteron/blob/main/CONTRIBUTING.md) for more information on how to get involved.

## License

This project is dual-licensed under either the [MIT License](https://opensource.org/licenses/MIT) or [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). You may choose which one to use.

## Links

- [Documentation on docs.rs](https://docs.rs/rusteron-media-driver/)
- [API Reference on github](https://gsrxyz.github.io/rusteron/rusteron_media_driver)
- [GitHub Repository](https://github.com/gsrxyz/rusteron)

Feel free to reach out with any questions or suggestions via GitHub Issues!

