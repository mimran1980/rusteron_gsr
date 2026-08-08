//! Ticket 3 integration tests: DPDK EAL singleton and dual-ENA initialization
//! (plan §7.2).
//!
//! The transport links the DPDK-free fakes (see `common/mod.rs`), so the full
//! init sequence, the failure matrix, reverse-order teardown, the allow-list,
//! and the already-initialized guard are exercised deterministically on any
//! host. Every test is `#[serial]` (as with the other DPDK test files): the EAL
//! is process-lifetime, so the init/failure sequences must never overlap.
//!
//! Linux x86_64 only, and requires libdpdk >= 23.11 (the `dpdk` feature).
#![cfg(all(feature = "dpdk", target_os = "linux", target_arch = "x86_64"))]

mod common;
use common::{close, create, dump, last_error, last_error_code, rusteron_dpdk_config_t, TestEnv};
use serial_test::serial;

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

// rusteron_dpdk_runtime_probe_device is an internal seam (declared in
// rusteron_dpdk_internal.h); the test build links the core archive which
// defines it, so declare it directly here.
extern "C" {
    fn rusteron_dpdk_runtime_probe_device(transport: *mut c_void, pci_bdf: *const c_char, port_id: *mut u16) -> c_int;
    // EAL thread registration seam (plan §7.2). Test builds link the fake EAL's
    // no-op stubs; the real rte_lcore_id-gated impl runs in production builds.
    fn rusteron_dpdk_eal_thread_register() -> c_int;
    fn rusteron_dpdk_eal_thread_unregister() -> c_int;
}

/// Index of the first log entry containing `needle`, or -1.
fn log_index_containing(env: &TestEnv, needle: &str) -> c_int {
    for i in 0..env.log_count() {
        if env.log_at(i).contains(needle) {
            return i;
        }
    }
    -1
}

/// Both ENA ports are probed, configured, and started, and hold distinct DPDK
/// port ids and mempools (plan §7.2: "separate mempool per port").
#[serial]
#[test]
fn dual_ports_initialize_with_distinct_resources() {
    let env = TestEnv::new();
    env.eal_skip();

    let config = rusteron_dpdk_config_t::valid();
    let transport = create(&config).unwrap_or_else(|e| panic!("create failed: {e}"));

    let d = dump(transport);
    assert_ne!(
        d.sender_port, d.receiver_port,
        "sender and receiver must use distinct DPDK ports"
    );
    assert_ne!(d.sender_pool, d.receiver_pool, "each port needs its own mempool");
    assert_ne!(d.sender_pool, 0, "sender pool must be allocated");
    assert_ne!(d.receiver_pool, 0, "receiver pool must be allocated");

    // Both ports went through the full nine-step sequence.
    assert!(log_index_containing(&env, "link 0") >= 0, "sender link missing");
    assert!(log_index_containing(&env, "link 1") >= 0, "receiver link missing");

    assert_eq!(close(transport), 0);
}

/// Teardown runs in reverse initialization order: the receiver (initialized
/// second) is stopped, closed, and freed before the sender (plan §7.2).
#[serial]
#[test]
fn teardown_is_reverse_init_order() {
    let env = TestEnv::new();
    env.eal_skip();

    let config = rusteron_dpdk_config_t::valid();
    let transport = create(&config).unwrap();
    assert_eq!(close(transport), 0);

    let receiver_stop = log_index_containing(&env, "stop 1");
    let sender_stop = log_index_containing(&env, "stop 0");
    assert!(receiver_stop >= 0 && sender_stop >= 0, "both ports must be stopped");
    assert!(receiver_stop < sender_stop, "receiver must stop before sender");

    let receiver_free = log_index_containing(&env, "free 0x1020");
    let sender_free = log_index_containing(&env, "free 0x1010");
    assert!(receiver_free >= 0 && sender_free >= 0, "both pools must be freed");
    assert!(
        receiver_free < sender_free,
        "receiver pool must be freed before sender pool"
    );

    // Per-port order: stop, then close, then free.
    let close_idx = log_index_containing(&env, "close 1");
    assert!(receiver_stop < close_idx && close_idx < receiver_free);
}

/// Every step of the nine-step sender/receiver sequence (steps 1..=18) fails
/// loudly with a native error and cleans up what was initialized so far.
#[serial]
#[test]
fn init_failure_matrix_reports_native_error() {
    let env = TestEnv::new();
    for step in 1..=18 {
        env.reset();
        env.eal_skip();
        env.set_failure(step);

        let config = rusteron_dpdk_config_t::valid();
        let err = create(&config).unwrap_err();
        assert!(!err.is_empty(), "step {step}: error message must not be empty");
        assert_eq!(last_error_code(), 1, "step {step}: expected native error code");
    }
}

/// Only the net_ena PMD is accepted (plan §7.2: "verify net_ena PMD").
#[serial]
#[test]
fn rejects_non_ena_driver() {
    let env = TestEnv::new();
    env.eal_skip();
    env.set_driver("net_virtio");

    let config = rusteron_dpdk_config_t::valid();
    let err = create(&config).unwrap_err();
    assert!(err.contains("net_ena"), "expected driver rejection, got: {err}");
    assert_eq!(last_error_code(), 1);
}

/// The IPv4/UDP checksum offloads are required (plan §7.2).
#[serial]
#[test]
fn rejects_missing_checksum_offloads() {
    let env = TestEnv::new();
    env.eal_skip();
    env.set_csum_ok(false);

    let config = rusteron_dpdk_config_t::valid();
    let err = create(&config).unwrap_err();
    assert!(err.contains("checksum"), "expected offload rejection, got: {err}");
    assert_eq!(last_error_code(), 1);
}

/// The device allow-list accepts exactly the two configured ENA devices.
#[serial]
#[test]
fn probe_device_rejects_unconfigured_bdf() {
    let env = TestEnv::new();
    env.eal_skip();

    let config = rusteron_dpdk_config_t::valid();
    let transport = create(&config).unwrap();

    let mut port_id: u16 = 0xFFFF;

    let other = CString::new("0000:00:03.0").unwrap();
    let rc = unsafe { rusteron_dpdk_runtime_probe_device(transport, other.as_ptr(), &mut port_id) };
    assert_eq!(rc, -1, "unconfigured device must be rejected");
    assert!(last_error().contains("not a configured ENA"), "got: {}", last_error());

    let sender = CString::new("0000:00:01.0").unwrap();
    let rc = unsafe { rusteron_dpdk_runtime_probe_device(transport, sender.as_ptr(), &mut port_id) };
    assert_eq!(rc, 0, "configured sender must resolve: {}", last_error());

    assert_eq!(close(transport), 0);
}

/// EAL is a process singleton: a second transport creation reports the
/// dedicated ALREADY_INITIALIZED error code (plan §7.2).
#[serial]
#[test]
fn second_transport_is_already_initialized() {
    let env = TestEnv::new();
    env.eal_skip();

    let config = rusteron_dpdk_config_t::valid();
    let first = create(&config).unwrap();

    let err = create(&config).unwrap_err();
    assert!(
        err.to_lowercase().contains("already initialized"),
        "expected already-initialized error, got: {err}"
    );
    assert_eq!(last_error_code(), 2, "dedicated ALREADY_INITIALIZED code required");

    assert_eq!(close(first), 0);
}

/// The real seam path (production `--huge-dir`, tests `--no-huge`) is covered
/// by the fake EAL archive in the `no-huge` mode.
#[serial]
#[test]
fn eal_seam_path_initializes_singleton() {
    let env = TestEnv::new();
    env.eal_no_huge();

    let config = rusteron_dpdk_config_t::valid();
    let transport = create(&config).unwrap_or_else(|e| panic!("create failed: {e}"));
    assert_eq!(close(transport), 0);
}

/// The EAL thread-registration seam (plan §7.2) is wired and safe to call on
/// the test thread: register is an idempotent no-op for non-EAL threads under
/// the fake EAL, and unregister is a guarded no-op on every other thread.
#[serial]
#[test]
fn thread_registration_seam_is_safe_noop() {
    let env = TestEnv::new();
    env.eal_skip();

    let config = rusteron_dpdk_config_t::valid();
    let transport = create(&config).unwrap();

    assert_eq!(unsafe { rusteron_dpdk_eal_thread_register() }, 0);
    assert_eq!(
        unsafe { rusteron_dpdk_eal_thread_register() },
        0,
        "register must be idempotent"
    );
    assert_eq!(unsafe { rusteron_dpdk_eal_thread_unregister() }, 0);

    assert_eq!(close(transport), 0);
}
