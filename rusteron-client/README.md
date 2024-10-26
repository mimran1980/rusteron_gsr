# rusteron-client

**rusteron-client** is a core component of the **rusteron** project, providing client functionalities to interact with the Aeron messaging system in a Rust environment. It enables Rust developers to leverage Aeron's high-performance, low-latency communication protocols.

## Overview

The **rusteron-client** module acts as a Rust wrapper around the Aeron C client API. It offers functions for establishing connections, publishing messages, and subscribing to data streams, allowing seamless communication between distributed applications. Since it is built on top of Aeron's C bindings, this library operates in an `unsafe` context, requiring extra care from developers to ensure correctness.

> **Note**: Since this module leverages Aeron C bindings, it is inherently unsafe and should be used with caution. Incorrect usage can lead to undefined behavior, such as segmentation faults.

## Features

- **Client Initialization**: Set up an Aeron client in Rust.
- **Publication**: Send messages to various Aeron channels.
- **Subscription**: Receive messages from Aeron channels.
- **Callbacks**: Handle events such as new publications, new subscriptions, and errors.

## Installation

Add the following to your `Cargo.toml` file to include **rusteron-client**:

```toml
[dependencies]
rusteron-client = "0.1"
```

Ensure you have also set up the necessary Aeron C libraries required by **rusteron-client**.

## Usage Example

```rust ,no_run
use rusteron_client::*;
use rusteron_media_driver::{AeronDriverContext, AeronDriver};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start embedded media driver for testing purposes
    let media_driver_ctx = AeronDriverContext::new()?;
    let (stop, driver_handle) = AeronDriver::launch_embedded(media_driver_ctx.clone(), false);
    let stop3 = stop.clone();

    let ctx = AeronContext::new()?;
    ctx.set_dir(media_driver_ctx.get_dir())?;
    let aeron = Aeron::new(ctx)?;
    aeron.start()?;
    
    // Set up the publication
    let publisher = aeron
        .async_add_publication("aeron:ipc", 123)?
        .poll_blocking(Duration::from_secs(5))?;
    let publisher2 = publisher.clone();

    // Start publishing messages
    let message = "Hello, Aeron!".as_bytes();
    std::thread::spawn(move || {
        while !stop.load(Ordering::Acquire) {
            if publisher.offer(message, Handlers::no_reserved_value_supplier_handler()) > 0 {
                println!("Sent message: Hello, Aeron!");
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });
    
    // Set up the publication with `try_claim`
    let string_len = 156;

    std::thread::spawn(move || {
        let buffer = AeronBufferClaim::default();
        let binding = "1".repeat(string_len);
        let msg = binding.as_bytes();
        while !stop3.load(Ordering::Acquire) {
            let result = publisher2.try_claim(string_len, &buffer);

            if result < msg.len() as i64 {
                eprintln!("ERROR: failed to send message {:?}", AeronCError::from_code(result as i32));
            } else {
                buffer.data_mut().write_all(&msg).unwrap();
                buffer.commit().unwrap();
                println!("Sent message [result={}]", result);
            }
        }
    });

    // Set up the subscription
    let subscription = aeron
        .async_add_subscription("aeron:ipc", 123,                
                                Handlers::no_available_image_handler(),
                                Handlers::no_unavailable_image_handler())?
        .poll_blocking(Duration::from_secs(5))?;

    let string_len = 156;
    let closure = AeronFragmentHandlerClosure::from(move |msg: Vec<u8>, header: AeronHeader| {
        println!(
            "Received a message from Aeron [position={:?}], msg length: {}",
            header.position(),
            msg.len()
        );
    });
    let closure = Handler::leak(closure);

    // Start receiving messages
    loop {
        subscription.poll(Some(&closure), 128)?;
    }


    stop.store(true, Ordering::SeqCst);
    driver_handle.join().unwrap();
    Ok(())
}
```

## Safety Considerations

Since **rusteron-client** relies on Aeron C bindings, it involves `unsafe` Rust code. Users must ensure:

- Resources are properly managed (e.g., not using a publisher after the Aeron context is closed).
- Proper synchronization when accessing shared data in a multithreaded environment.

Failing to uphold these safety measures can lead to crashes or undefined behavior.

## Contributing

Contributions are welcome! Please feel free to open issues, submit pull requests, or suggest new features. We're particularly interested in:

- Feedback on API usability.
- Bug reports and feature requests.
- Documentation improvements.

If you wish to contribute, refer to our [contributing guidelines](https://github.com/mimran1980/rusteron/blob/main/CONTRIBUTING.md).

## License

This project is dual-licensed under either the [MIT License](https://opensource.org/licenses/MIT) or [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). You may choose which one to use.

## Links

- [Documentation on docs.rs](https://docs.rs/rusteron-client/)
- [GitHub Repository](https://github.com/mimran1980/rusteron)

Feel free to reach out with any questions or suggestions via GitHub Issues!

