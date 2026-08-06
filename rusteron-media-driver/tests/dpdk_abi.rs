//! ABI smoke test for the native DPDK transport (Ticket 1).
//!
//! Verifies the stable Rust/native ABI contract in
//! `native/dpdk/rusteron_dpdk_transport.h`: the config struct layout guard, the
//! five exported functions, and that the Aeron transport binding table is fully
//! populated with all twelve callbacks under the `rusteron-dpdk-ena` name.
//!
//! Linux x86_64 only, and requires libdpdk >= 23.11 (the `dpdk` feature).
#![cfg(all(feature = "dpdk", target_os = "linux", target_arch = "x86_64"))]

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

// Cargo forwards build-script `rustc-link-lib` directives to the package lib
// and to dependents (via `links` metadata) but NOT to same-package integration
// tests; it forwards search paths and link-args only. So register the two
// native libraries this test links directly:
//   - `aeron_driver`  dylib  for aeron_driver_context_init/close (bindings.rs)
//   - `rusteron_dpdk` static for the five rusteron_dpdk_* ABI functions
// The static shim currently references no DPDK symbols (Ticket 1 stubs), so no
// `rte_*` libraries are needed at this stage.
#[link(name = "aeron_driver")]
#[link(name = "rusteron_dpdk", kind = "static")]
extern "C" {}

// Real Aeron types/functions from the generated bindings so we can exercise
// `install` against a genuine driver context.
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[repr(C)]
#[derive(Debug)]
struct rusteron_dpdk_config_t {
    struct_size: u32,
    file_prefix: [c_char; 65],
    hugepage_dir: [c_char; 4096],
    sender_pci: [c_char; 16],
    sender_ipv4: [c_char; 16],
    sender_prefix_len: u8,
    sender_gateway: [c_char; 16],
    receiver_pci: [c_char; 16],
    receiver_ipv4: [c_char; 16],
    receiver_prefix_len: u8,
    receiver_gateway: [c_char; 16],
    rx_descriptors: u16,
    tx_descriptors: u16,
    mbufs_per_port: u32,
    mempool_cache: u16,
    burst_size: u16,
    max_aeron_mtu: usize,
}

impl rusteron_dpdk_config_t {
    fn valid() -> Self {
        let mut c = Self {
            struct_size: 0, // filled below
            file_prefix: [0; 65],
            hugepage_dir: [0; 4096],
            sender_pci: [0; 16],
            sender_ipv4: [0; 16],
            sender_prefix_len: 24,
            sender_gateway: [0; 16],
            receiver_pci: [0; 16],
            receiver_ipv4: [0; 16],
            receiver_prefix_len: 24,
            receiver_gateway: [0; 16],
            rx_descriptors: 1024,
            tx_descriptors: 1024,
            mbufs_per_port: 8192,
            mempool_cache: 256,
            burst_size: 64,
            max_aeron_mtu: 1472,
        };
        fill_cstr(&mut c.file_prefix, "rusteron-ena");
        fill_cstr(&mut c.hugepage_dir, "/dev/hugepages");
        fill_cstr(&mut c.sender_pci, "0000:00:01.0");
        fill_cstr(&mut c.sender_ipv4, "10.0.0.1");
        fill_cstr(&mut c.sender_gateway, "10.0.0.254");
        fill_cstr(&mut c.receiver_pci, "0000:00:02.0");
        fill_cstr(&mut c.receiver_ipv4, "10.0.1.1");
        fill_cstr(&mut c.receiver_gateway, "10.0.1.254");
        c.struct_size = std::mem::size_of::<Self>() as u32;
        c
    }
}

fn fill_cstr(buf: &mut [c_char], value: &str) {
    let bytes = value.as_bytes();
    assert!(bytes.len() < buf.len(), "string too long for buffer");
    for (i, b) in bytes.iter().enumerate() {
        buf[i] = *b as c_char;
    }
    buf[bytes.len()] = 0;
}

fn last_error() -> String {
    let ptr = unsafe { rusteron_dpdk_last_error() };
    if ptr.is_null() {
        return "<null>".to_string();
    }
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
}

extern "C" {
    fn rusteron_dpdk_transport_create(
        config: *const rusteron_dpdk_config_t,
        transport: *mut *mut c_void,
    ) -> c_int;
    fn rusteron_dpdk_transport_install(
        transport: *mut c_void,
        context: *mut aeron_driver_context_t,
    ) -> c_int;
    fn rusteron_dpdk_transport_close(transport: *mut c_void) -> c_int;
    fn rusteron_dpdk_transport_bindings() -> *mut aeron_udp_channel_transport_bindings_stct;
    fn rusteron_dpdk_last_error() -> *const c_char;
    // aeron_driver_context_init / aeron_driver_context_close come from the
    // bindings.rs include above.
}

#[test]
fn all_twelve_binding_callbacks_populated() {
    let bindings = unsafe { rusteron_dpdk_transport_bindings() };
    assert!(!bindings.is_null(), "bindings must be non-NULL");

    let b = unsafe { &*bindings };
    assert!(b.init_func.is_some());
    assert!(b.reconnect_func.is_some());
    assert!(b.close_func.is_some());
    assert!(b.recvmmsg_func.is_some());
    assert!(b.send_func.is_some());
    assert!(b.get_so_rcvbuf_func.is_some());
    assert!(b.bind_addr_and_port_func.is_some());
    assert!(b.poller_init_func.is_some());
    assert!(b.poller_close_func.is_some());
    assert!(b.poller_add_func.is_some());
    assert!(b.poller_remove_func.is_some());
    assert!(b.poller_poll_func.is_some());

    let name = unsafe { CStr::from_ptr(b.meta_info.name) };
    assert_eq!(name.to_str().unwrap(), "rusteron-dpdk-ena");
    let typ = unsafe { CStr::from_ptr(b.meta_info.type_) };
    assert_eq!(typ.to_str().unwrap(), "media");
    assert!(!b.meta_info.source_symbol.is_null(), "source_symbol must be set");
}

#[test]
fn create_close_roundtrip() {
    let config = rusteron_dpdk_config_t::valid();
    let mut transport: *mut c_void = ptr::null_mut();
    let rc = unsafe { rusteron_dpdk_transport_create(&config, &mut transport) };
    assert_eq!(rc, 0, "create failed: {}", last_error());
    assert!(!transport.is_null());

    let rc = unsafe { rusteron_dpdk_transport_close(transport) };
    assert_eq!(rc, 0);
}

#[test]
fn create_rejects_struct_size_mismatch() {
    let mut config = rusteron_dpdk_config_t::valid();
    config.struct_size = 0; // corrupt the layout guard
    let mut transport: *mut c_void = ptr::null_mut();
    let rc = unsafe { rusteron_dpdk_transport_create(&config, &mut transport) };
    assert_eq!(rc, -1, "mismatched struct_size must be rejected");
    assert!(transport.is_null());
    assert!(
        last_error().contains("struct_size"),
        "expected struct_size error, got: {}",
        last_error()
    );
}

#[test]
fn create_rejects_bad_burst_size() {
    let mut config = rusteron_dpdk_config_t::valid();
    config.burst_size = 0;
    let mut transport: *mut c_void = ptr::null_mut();
    let rc = unsafe { rusteron_dpdk_transport_create(&config, &mut transport) };
    assert_eq!(rc, -1);
    assert!(last_error().contains("burst_size"), "got: {}", last_error());
}

#[test]
fn install_replaces_context_transport_bindings() {
    let mut context: *mut aeron_driver_context_t = ptr::null_mut();
    let rc = unsafe { aeron_driver_context_init(&mut context) };
    assert_eq!(rc, 0, "context init failed");
    assert!(!context.is_null());

    let config = rusteron_dpdk_config_t::valid();
    let mut transport: *mut c_void = ptr::null_mut();
    assert_eq!(unsafe { rusteron_dpdk_transport_create(&config, &mut transport) }, 0);

    let rc = unsafe { rusteron_dpdk_transport_install(transport, context) };
    assert_eq!(rc, 0, "install failed: {}", last_error());

    let ctx = unsafe { &*context };
    let expected = unsafe { rusteron_dpdk_transport_bindings() } as *const _;
    assert!(
        std::ptr::eq(ctx.udp_channel_transport_bindings as *const _, expected),
        "context transport bindings must point at the DPDK table"
    );

    unsafe { rusteron_dpdk_transport_close(transport) };
    unsafe { aeron_driver_context_close(context) };
}

#[test]
fn last_error_defaults_to_no_error() {
    assert_eq!(last_error(), "no error");
}
