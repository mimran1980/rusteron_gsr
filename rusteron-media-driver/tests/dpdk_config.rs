//! Ticket 2 integration tests: typed configuration, environment parsing, and
//! lifetime ownership (plan §5.1/§5.2/§6).
//!
//! Pure config/env/error behaviour runs on every platform with the default
//! features; the full install/lifetime path is `#[cfg(feature = "dpdk")]`
//! (Linux x86_64 in the Docker verification image).
//!
//! These tests mutate process-global env vars, so every test is `#[serial]`
//! (file-locked across the whole workspace, as with the other DPDK test files).

use serial_test::serial;

use rusteron_media_driver::dpdk::config::DpdkTransportConfig;
use rusteron_media_driver::dpdk::env;
use rusteron_media_driver::dpdk::error::DpdkError;
use rusteron_media_driver::dpdk::{configure_media_transport_from_env, DpdkPortConfig, DpdkTransport};
use rusteron_media_driver::AeronDriverContext;

// Native libraries and the DPDK-free fakes are declared in `common/mod.rs`
// (cargo forwards build-script link-libs to the lib, not to same-package
// integration tests). Feature-gated install tests drive the transport under
// the skip-EAL test mode.
mod common;
#[cfg(feature = "dpdk")]
use common::TestEnv;

const ENV_VARS: &[&str] = &[
    "RUSTERON_MEDIA_DRIVER_TRANSPORT",
    "RUSTERON_DPDK_FILE_PREFIX",
    "RUSTERON_DPDK_SENDER_PCI",
    "RUSTERON_DPDK_SENDER_IPV4_CIDR",
    "RUSTERON_DPDK_SENDER_GATEWAY",
    "RUSTERON_DPDK_RECEIVER_PCI",
    "RUSTERON_DPDK_RECEIVER_IPV4_CIDR",
    "RUSTERON_DPDK_RECEIVER_GATEWAY",
    "RUSTERON_DPDK_HUGE_DIR",
    "RUSTERON_DPDK_RX_DESCRIPTORS",
    "RUSTERON_DPDK_TX_DESCRIPTORS",
    "RUSTERON_DPDK_MBUFS_PER_PORT",
    "RUSTERON_DPDK_MEMPOOL_CACHE",
    "RUSTERON_DPDK_BURST_SIZE",
    "RUSTERON_DPDK_MAX_AERON_MTU",
];

fn clear_env() {
    for var in ENV_VARS {
        std::env::remove_var(var);
    }
}

/// A valid configuration matching the plan's recommended standalone settings.
fn canonical() -> DpdkTransportConfig {
    DpdkTransportConfig {
        sender: DpdkPortConfig {
            pci_address: "0000:00:01.0".into(),
            local_ipv4: "10.0.0.1".parse().unwrap(),
            prefix_len: 24,
            gateway_ipv4: "10.0.0.254".parse().unwrap(),
        },
        receiver: DpdkPortConfig {
            pci_address: "0000:00:02.0".into(),
            local_ipv4: "10.0.1.1".parse().unwrap(),
            prefix_len: 24,
            gateway_ipv4: "10.0.1.254".parse().unwrap(),
        },
        file_prefix: "rusteron-ena".into(),
        test_vdev: false,
        hugepage_dir: "/dev/hugepages".into(),
        rx_descriptors: 1024,
        tx_descriptors: 1024,
        mbufs_per_port: 65536,
        mempool_cache: 256,
        burst_size: 32,
        max_aeron_mtu: 1408,
    }
}

fn set_env_from_canonical() {
    std::env::set_var("RUSTERON_DPDK_FILE_PREFIX", "rusteron-ena");
    std::env::set_var("RUSTERON_DPDK_SENDER_PCI", "0000:00:01.0");
    std::env::set_var("RUSTERON_DPDK_SENDER_IPV4_CIDR", "10.0.0.1/24");
    std::env::set_var("RUSTERON_DPDK_SENDER_GATEWAY", "10.0.0.254");
    std::env::set_var("RUSTERON_DPDK_RECEIVER_PCI", "0000:00:02.0");
    std::env::set_var("RUSTERON_DPDK_RECEIVER_IPV4_CIDR", "10.0.1.1/24");
    std::env::set_var("RUSTERON_DPDK_RECEIVER_GATEWAY", "10.0.1.254");
    std::env::set_var("RUSTERON_DPDK_HUGE_DIR", "/dev/hugepages");
    std::env::set_var("RUSTERON_DPDK_RX_DESCRIPTORS", "1024");
    std::env::set_var("RUSTERON_DPDK_TX_DESCRIPTORS", "1024");
    std::env::set_var("RUSTERON_DPDK_MBUFS_PER_PORT", "65536");
    std::env::set_var("RUSTERON_DPDK_MEMPOOL_CACHE", "256");
    std::env::set_var("RUSTERON_DPDK_BURST_SIZE", "32");
    std::env::set_var("RUSTERON_DPDK_MAX_AERON_MTU", "1408");
}

/// Assert a validation error that names `needle` (the offending field).
fn assert_invalid_config(config: &DpdkTransportConfig, needle: &str) {
    let err = config.validate().unwrap_err();
    assert!(
        matches!(err, DpdkError::InvalidConfiguration(_)),
        "expected InvalidConfiguration, got {err:?}"
    );
    let msg = err.to_string();
    assert!(msg.contains(needle), "message {msg:?} must name {needle:?}");
}

// --- Acceptance 1: typed and environment configurations resolve identically ---

#[serial]
#[test]
fn typed_and_env_configs_resolve_identical() {
    clear_env();
    set_env_from_canonical();
    assert_eq!(env::config_from_env().unwrap(), canonical());
}

// --- Validation rules (plan §6.5): each error names the offending field ------

#[serial]
#[test]
fn validation_rejects_bad_sender_pci() {
    let mut c = canonical();
    c.sender.pci_address = "00:01.0".into(); // missing domain
    assert_invalid_config(&c, "sender.pci_address");
}

// --- Test-only vdev selectors (plan §11.2): production stays PCI-only --------

#[serial]
#[test]
fn validation_rejects_vdev_name_outside_test_mode() {
    let mut c = canonical();
    c.sender.pci_address = "net_tap0".into();
    c.receiver.pci_address = "net_tap1".into();
    assert_invalid_config(&c, "sender.pci_address");
}

#[serial]
#[test]
fn validation_accepts_vdev_names_in_test_mode() {
    let mut c = canonical();
    c.test_vdev = true;
    c.sender.pci_address = "net_tap0".into();
    c.receiver.pci_address = "net_tap1".into();
    c.validate().unwrap();
}

#[serial]
#[test]
fn validation_rejects_bad_vdev_name_in_test_mode() {
    let mut c = canonical();
    c.test_vdev = true;
    c.sender.pci_address = "0000:00:01.0".into(); // PCI is always valid
    c.receiver.pci_address = "net_tap-with:colon".into(); // ':' would misdetect as PCI
    assert_invalid_config(&c, "receiver.pci_address");
}

#[serial]
#[test]
fn validation_rejects_duplicate_vdev_in_test_mode() {
    let mut c = canonical();
    c.test_vdev = true;
    c.sender.pci_address = "net_tap0".into();
    c.receiver.pci_address = "net_tap0".into();
    assert_invalid_config(&c, "must differ");
}

#[serial]
#[test]
fn validation_rejects_duplicate_pci() {
    let mut c = canonical();
    c.receiver.pci_address = c.sender.pci_address.clone();
    assert_invalid_config(&c, "must differ");
}

#[serial]
#[test]
fn validation_rejects_multicast_local_ip() {
    let mut c = canonical();
    c.sender.local_ipv4 = "224.0.0.1".parse().unwrap();
    assert_invalid_config(&c, "sender.local_ipv4");
}

#[serial]
#[test]
fn validation_rejects_broadcast_gateway() {
    let mut c = canonical();
    c.receiver.gateway_ipv4 = "255.255.255.255".parse().unwrap();
    assert_invalid_config(&c, "receiver.gateway_ipv4");
}

#[serial]
#[test]
fn validation_rejects_zero_prefix_len() {
    let mut c = canonical();
    c.receiver.prefix_len = 0;
    assert_invalid_config(&c, "receiver.prefix_len");
}

#[serial]
#[test]
fn validation_rejects_prefix_len_above_32() {
    let mut c = canonical();
    c.sender.prefix_len = 33;
    assert_invalid_config(&c, "sender.prefix_len");
}

#[serial]
#[test]
fn validation_rejects_gateway_outside_subnet() {
    let mut c = canonical();
    c.sender.gateway_ipv4 = "10.0.1.254".parse().unwrap(); // receiver's subnet
    assert_invalid_config(&c, "sender.gateway_ipv4");
}

#[serial]
#[test]
fn validation_rejects_bad_file_prefix() {
    let mut c = canonical();
    c.file_prefix = "rusteron/ena".into(); // '/' not allowed
    assert_invalid_config(&c, "file_prefix");
}

#[serial]
#[test]
fn validation_rejects_empty_file_prefix() {
    let mut c = canonical();
    c.file_prefix = String::new();
    assert_invalid_config(&c, "file_prefix");
}

#[serial]
#[test]
fn validation_rejects_relative_hugepage_dir() {
    let mut c = canonical();
    c.hugepage_dir = "hugepages".into();
    assert_invalid_config(&c, "hugepage_dir");
}

#[serial]
#[test]
fn validation_rejects_rx_descriptors_out_of_range() {
    let mut c = canonical();
    c.rx_descriptors = 63;
    assert_invalid_config(&c, "rx_descriptors");
}

#[serial]
#[test]
fn validation_rejects_tx_descriptors_out_of_range() {
    let mut c = canonical();
    c.tx_descriptors = 8193;
    assert_invalid_config(&c, "tx_descriptors");
}

#[serial]
#[test]
fn validation_rejects_zero_burst() {
    let mut c = canonical();
    c.burst_size = 0;
    assert_invalid_config(&c, "burst_size");
}

#[serial]
#[test]
fn validation_rejects_burst_above_256() {
    let mut c = canonical();
    c.burst_size = 257;
    assert_invalid_config(&c, "burst_size");
}

#[serial]
#[test]
fn validation_rejects_undersized_mbuf_pool() {
    let mut c = canonical();
    // rx + tx + 4*burst = 1024 + 1024 + 128 = 2176; one below is too small.
    c.mbufs_per_port = 2175;
    assert_invalid_config(&c, "mbufs_per_port");
}

#[serial]
#[test]
fn validation_rejects_unaligned_mtu() {
    let mut c = canonical();
    c.max_aeron_mtu = 1400; // not a multiple of 32
    assert_invalid_config(&c, "max_aeron_mtu");
}

#[serial]
#[test]
fn validation_rejects_mtu_above_1472() {
    let mut c = canonical();
    c.max_aeron_mtu = 1473;
    assert_invalid_config(&c, "max_aeron_mtu");
}

// --- Selector behaviour (plan §6.1) ------------------------------------------

#[serial]
#[test]
fn absent_selector_preserves_default() {
    clear_env();
    let ctx = AeronDriverContext::new().unwrap();
    let result = configure_media_transport_from_env(&ctx).unwrap();
    assert!(result.is_none(), "absent selector must preserve default behaviour");
}

#[serial]
#[test]
fn default_selector_preserves_default() {
    clear_env();
    std::env::set_var("RUSTERON_MEDIA_DRIVER_TRANSPORT", "default");
    let ctx = AeronDriverContext::new().unwrap();
    let result = configure_media_transport_from_env(&ctx).unwrap();
    assert!(result.is_none(), "`default` selector must preserve default behaviour");
}

#[serial]
#[test]
fn unknown_selector_is_invalid_environment() {
    clear_env();
    std::env::set_var("RUSTERON_MEDIA_DRIVER_TRANSPORT", "vanilla");
    let ctx = AeronDriverContext::new().unwrap();
    let err = configure_media_transport_from_env(&ctx).unwrap_err();
    assert!(matches!(err, DpdkError::InvalidEnvironment(_)), "got {err:?}");
    assert!(err.to_string().contains("RUSTERON_MEDIA_DRIVER_TRANSPORT"));
}

#[serial]
#[test]
fn empty_selector_is_invalid_environment() {
    clear_env();
    std::env::set_var("RUSTERON_MEDIA_DRIVER_TRANSPORT", "");
    let ctx = AeronDriverContext::new().unwrap();
    let err = configure_media_transport_from_env(&ctx).unwrap_err();
    assert!(matches!(err, DpdkError::InvalidEnvironment(_)), "got {err:?}");
}

// --- Environment parsing failures (plan §6.2/§6.3) ---------------------------

#[serial]
#[test]
fn config_from_env_reports_missing_variable() {
    clear_env();
    set_env_from_canonical();
    std::env::remove_var("RUSTERON_DPDK_SENDER_PCI");
    let err = env::config_from_env().unwrap_err();
    assert!(matches!(err, DpdkError::MissingEnvironment(_)), "got {err:?}");
    assert!(err.to_string().contains("RUSTERON_DPDK_SENDER_PCI"));
}

#[serial]
#[test]
fn config_from_env_reports_cidr_without_prefix() {
    clear_env();
    set_env_from_canonical();
    std::env::set_var("RUSTERON_DPDK_SENDER_IPV4_CIDR", "10.0.0.1");
    let err = env::config_from_env().unwrap_err();
    assert!(matches!(err, DpdkError::InvalidEnvironment(_)), "got {err:?}");
    assert!(err.to_string().contains("SENDER_IPV4_CIDR"));
}

#[serial]
#[test]
fn config_from_env_reports_bad_prefix_number() {
    clear_env();
    set_env_from_canonical();
    std::env::set_var("RUSTERON_DPDK_SENDER_IPV4_CIDR", "10.0.0.1/33");
    let err = env::config_from_env().unwrap_err();
    assert!(matches!(err, DpdkError::InvalidEnvironment(_)), "got {err:?}");
    assert!(err.to_string().contains("prefix"));
}

#[serial]
#[test]
fn config_from_env_reports_bad_gateway() {
    clear_env();
    set_env_from_canonical();
    std::env::set_var("RUSTERON_DPDK_RECEIVER_GATEWAY", "not-an-ip");
    let err = env::config_from_env().unwrap_err();
    assert!(matches!(err, DpdkError::InvalidEnvironment(_)), "got {err:?}");
    assert!(err.to_string().contains("RECEIVER_GATEWAY"));
}

#[serial]
#[test]
fn config_from_env_reports_bad_number_default() {
    clear_env();
    set_env_from_canonical();
    std::env::set_var("RUSTERON_DPDK_BURST_SIZE", "lots");
    let err = env::config_from_env().unwrap_err();
    assert!(matches!(err, DpdkError::InvalidEnvironment(_)), "got {err:?}");
    assert!(err.to_string().contains("RUSTERON_DPDK_BURST_SIZE"));
}

#[serial]
#[test]
fn env_defaults_match_plan() {
    clear_env();
    set_env_from_canonical();
    for var in [
        "RUSTERON_DPDK_HUGE_DIR",
        "RUSTERON_DPDK_RX_DESCRIPTORS",
        "RUSTERON_DPDK_TX_DESCRIPTORS",
        "RUSTERON_DPDK_MBUFS_PER_PORT",
        "RUSTERON_DPDK_MEMPOOL_CACHE",
        "RUSTERON_DPDK_BURST_SIZE",
        "RUSTERON_DPDK_MAX_AERON_MTU",
    ] {
        std::env::remove_var(var);
    }
    let c = env::config_from_env().unwrap();
    assert_eq!(c.hugepage_dir, std::path::PathBuf::from("/dev/hugepages"));
    assert_eq!(c.rx_descriptors, 1024);
    assert_eq!(c.tx_descriptors, 1024);
    assert_eq!(c.mbufs_per_port, 65536);
    assert_eq!(c.mempool_cache, 256);
    assert_eq!(c.burst_size, 32);
    assert_eq!(c.max_aeron_mtu, 1408);
}

// --- Feature disabled (runs everywhere without `dpdk`) ------------------------

#[cfg(not(feature = "dpdk"))]
#[serial]
#[test]
fn dpdk_ena_without_feature_is_feature_disabled() {
    clear_env();
    std::env::set_var("RUSTERON_MEDIA_DRIVER_TRANSPORT", "dpdk-ena");
    set_env_from_canonical(); // env is fully set; the feature must still win
    let ctx = AeronDriverContext::new().unwrap();
    let err = configure_media_transport_from_env(&ctx).unwrap_err();
    assert!(matches!(err, DpdkError::FeatureDisabled), "got {err:?}");
}

#[cfg(not(feature = "dpdk"))]
#[serial]
#[test]
fn install_without_feature_is_feature_disabled() {
    clear_env();
    let ctx = AeronDriverContext::new().unwrap();
    let err = DpdkTransport::install(&ctx, canonical()).unwrap_err();
    assert!(matches!(err, DpdkError::FeatureDisabled), "got {err:?}");
}

// --- Install + lifetime (plan §5.2, feature-gated) ----------------------------

/// A context meeting the plan §6.4 required state (see §6.4 recommended values).
#[cfg(feature = "dpdk")]
fn configured_context() -> AeronDriverContext {
    use rusteron_media_driver::AeronIdleStrategyKind;
    let ctx = AeronDriverContext::new().unwrap();
    ctx.set_sender_cpu_affinity(1).unwrap();
    ctx.set_receiver_cpu_affinity(2).unwrap();
    ctx.set_sender_idle_strategy_kind(AeronIdleStrategyKind::BusySpin)
        .unwrap();
    ctx.set_receiver_idle_strategy_kind(AeronIdleStrategyKind::BusySpin)
        .unwrap();
    ctx.set_sender_wildcard_port_range(20000, 20999).unwrap();
    ctx.set_receiver_wildcard_port_range(21000, 21999).unwrap();
    ctx.set_mtu_length(1408).unwrap();
    ctx
}

#[cfg(feature = "dpdk")]
#[serial]
#[test]
fn install_succeeds_on_configured_context() {
    let env = TestEnv::new();
    env.eal_skip();
    let ctx = configured_context();
    let guard = DpdkTransport::install(&ctx, canonical()).unwrap();
    drop(guard); // guard drop alone must not stop DPDK (context holds a clone)
}

#[cfg(feature = "dpdk")]
#[serial]
#[test]
fn install_rejects_misconfigured_context() {
    let _env = TestEnv::new();
    // Default context: sender/receiver cpu affinity = -1 (§6.4 requirement fails).
    let ctx = AeronDriverContext::new().unwrap();
    let err = DpdkTransport::install(&ctx, canonical()).unwrap_err();
    assert!(matches!(err, DpdkError::InvalidConfiguration(_)), "got {err:?}");
    assert!(err.to_string().contains("cpu affinity"), "got {err}");
}

#[cfg(feature = "dpdk")]
#[serial]
#[test]
fn dropping_caller_guard_keeps_context_owned_transport_alive() {
    let env = TestEnv::new();
    env.eal_skip();
    let ctx = configured_context();
    let guard = DpdkTransport::install(&ctx, canonical()).unwrap();
    drop(guard);

    // The context retained a clone in its dependency graph, so the native
    // transport must still be alive after the caller's guard is dropped. The
    // `Arc`-backed close-on-last-drop guarantee means native close runs only
    // once the context is gone too.
    let retained = ctx.get_dependency::<DpdkTransport>();
    assert!(
        retained.is_some(),
        "context must retain a transport clone after the caller's guard is dropped"
    );
}
