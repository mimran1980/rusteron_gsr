# rusteron-rb

**rusteron-rb** is a core component of the **rusteron** project, providing ring buffer and broadcast functionalities to interact with the Aeron messaging system in a Rust environment. It enables Rust developers to leverage Aeron's high-performance, low-latency communication protocols.

## Overview

The **rusteron-rb** module acts as a Rust wrapper around the Aeron C ring buffer API. It offers functions for establishing connections, transmitting and receiving messages, and broadcasting data streams, allowing seamless communication between distributed applications. Since it is built on top of Aeron's C bindings, this library operates in an `unsafe` context, requiring extra care from developers to ensure correctness.

> **Note**: Since this module leverages Aeron C bindings, it is inherently unsafe and should be used with caution. Incorrect usage can lead to undefined behavior, such as segmentation faults.

## Features

- **Broadcast Receiver**: Receive messages from Aeron broadcast channels using `AeronBroadcastReceiver`.
- **Broadcast Transmitter**: Send messages to Aeron broadcast channels using `AeronBroadcastTransmitter`.
- **MPSC Ring Buffer**: Multi-producer, single-consumer ring buffer implementation using `AeronMpscRb`.
- **SPSC Ring Buffer**: Single-producer, single-consumer ring buffer implementation using `AeronSpscRb`.
- **Automatic Resource Management**: Resources are automatically managed, ensuring proper cleanup and efficient memory usage.

## Installation

Add the following to your `Cargo.toml` file to include **rusteron-rb**:

dynamic lib
```toml
[dependencies]
rusteron-rb = "0.1"
```

static lib
```toml
[dependencies]
rusteron-rb = { version = "0.1", features = ["static"] }
```

Ensure you have also set up the necessary Aeron C libraries required by **rusteron-rb**.

## Usage Example

Here is a basic example demonstrating the initialization and usage of `AeronBroadcastReceiver` and `AeronBroadcastTransmitter`:

```rust
use rusteron_rb::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    Ok(())
}
```

## Safety Considerations

Since **rusteron-rb** relies on Aeron C bindings, it involves `unsafe` Rust code. Users must ensure:

- Resources are properly managed.
- Proper synchronization when accessing shared data in a multi-threaded environment.

Failing to uphold these safety measures can lead to crashes or undefined behavior.

## Contributing

Contributions are welcome! Please feel free to open issues, submit pull requests, or suggest new features. We're particularly interested in:

- Feedback on API usability.
- Bug reports and feature requests.
- Documentation improvements.

If you wish to contribute, refer to our [contributing guidelines](https://github.com/mimran1980/rusteron/blob/main/CONTRIBUTING.md).

## License

This project is dual-licensed under either the [MIT License](https://opensource.org/licenses/MIT) or the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). You may choose which one to use.

## Links

- [Documentation on docs.rs](https://docs.rs/rusteron-rb/)
- [API Reference on GitHub](https://mimran1980.github.io/rusteron/rusteron_rb)
- [GitHub Repository](https://github.com/mimran1980/rusteron)

Feel free to reach out with any questions or suggestions via GitHub Issues!