# rusteron-client

`rusteron-client` is a core component of the [Rusteron](https://github.com/gsrxyz/rusteron) project.  
It provides a Rust wrapper around the Aeron C client API, enabling high-performance, low-latency communication in distributed systems built with Rust.

This crate supports publishing, subscribing, and managing Aeron resources, while exposing a flexible and idiomatic interface over `unsafe` C bindings.  
Due to its reliance on raw FFI, developers must take care to manage resource lifetimes and concurrency correctly.

---

## Features

- **Client Setup** – Create and start an Aeron client using Rust.
- **Publications** – Send messages via `offer()` or `try_claim()`.
- **Subscriptions** – Poll for incoming messages and handle fragments.
- **Callbacks & Handlers** – React to driver events like availability, errors, and stream lifecycle changes.
- **Cloneable Wrappers** – All client types are cloneable and share ownership of the underlying C resources.
- **Automatic Resource Management** – Objects created with `.new()` automatically call `*_init` and `*_close`, where supported.
- **Result-Focused API** – Methods returning primitive C results return `Result<T, AeronCError>` for ergonomic error handling.
- **Efficient String Interop** – Inputs use `&CStr`, outputs return `&str`, giving developers precise allocation control.

---

## General Patterns

- **`new()` Initialization**: Automatically calls the corresponding `*_init` method.
- **Automatic Cleanup (Partial)**: When possible, `Drop` will invoke the appropriate `*_close` or `*_destroy` methods.
- **Manual Resource Responsibility**: For methods like `set_aeron()` or where lifetimes aren't managed internally, users are responsible for safety.
- **Handlers Must Be Leaked and Released**: Callbacks passed to the C layer require explicit memory management using `Handlers::leak(...)` and `Handlers::release(...)`.

---

## Handlers and Callbacks

Handlers allow users to customize responses to Aeron events (errors, image availability, etc). There are two ways to use them:

### 1. Implementing a Trait (Recommended)

This is the most performant and idiomatic approach.

```rust,no_ignore
use rusteron_client::*;

pub struct MyErrorHandler;

impl AeronErrorHandlerCallback for MyErrorHandler {
    fn handle_aeron_error_handler(&mut self, code: i32, msg: &str) {
        eprintln!("Aeron error ({}): {}", code, msg);
    }
}
