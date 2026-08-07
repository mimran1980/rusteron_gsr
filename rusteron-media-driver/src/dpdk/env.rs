//! Environment contract (plan §6): selector, required values, and defaults.

use crate::dpdk::config::{DpdkPortConfig, DpdkTransportConfig};
use crate::dpdk::error::DpdkError;
use std::env;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;

/// `RUSTERON_MEDIA_DRIVER_TRANSPORT` selector (plan §6.1).
const SELECTOR: &str = "RUSTERON_MEDIA_DRIVER_TRANSPORT";

/// Parsed transport selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Selector {
    /// `default` (or absent) — unchanged socket-based behaviour.
    Default,
    /// `dpdk-ena` — use the DPDK ENA kernel-bypass transport.
    DpdkEna,
}

/// Parse the transport selector (plan §6.1). Absent or `default` yields
/// [`Selector::Default`]; `dpdk-ena` yields [`Selector::DpdkEna`]; an empty or
/// unknown value is [`DpdkError::InvalidEnvironment`].
pub fn selector() -> Result<Selector, DpdkError> {
    match env::var(SELECTOR) {
        Err(_) => Ok(Selector::Default),
        Ok(v) if v == "default" => Ok(Selector::Default),
        Ok(v) if v == "dpdk-ena" => Ok(Selector::DpdkEna),
        Ok(v) => Err(DpdkError::InvalidEnvironment(format!(
            "{SELECTOR}={v:?} must be `default` or `dpdk-ena`"
        ))),
    }
}

/// Parse a full transport configuration from the environment (plan §6.2/§6.3).
///
/// Required values are `RUSTERON_DPDK_FILE_PREFIX`, `RUSTERON_DPDK_SENDER_PCI`,
/// `RUSTERON_DPDK_SENDER_IPV4_CIDR`, `RUSTERON_DPDK_SENDER_GATEWAY`, and the
/// `RECEIVER_` twins. Everything else falls back to the plan §6.3 defaults.
///
/// `RUSTERON_DPDK_TEST_VDEV=1` enables the test-only virtual-device mode (plan
/// §11.2): sender/receiver PCI may then be DPDK vdev names (e.g. `net_tap0`)
/// instead of ENA PCI BDFs. Production configurations never set it.
pub fn config_from_env() -> Result<DpdkTransportConfig, DpdkError> {
    let sender_cidr = required("RUSTERON_DPDK_SENDER_IPV4_CIDR")?;
    let receiver_cidr = required("RUSTERON_DPDK_RECEIVER_IPV4_CIDR")?;
    let (sender_addr, sender_prefix) = parse_cidr("sender", &sender_cidr)?;
    let (receiver_addr, receiver_prefix) = parse_cidr("receiver", &receiver_cidr)?;
    let test_vdev = env::var("RUSTERON_DPDK_TEST_VDEV").is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

    Ok(DpdkTransportConfig {
        sender: DpdkPortConfig {
            pci_address: required("RUSTERON_DPDK_SENDER_PCI")?,
            local_ipv4: sender_addr,
            prefix_len: sender_prefix,
            gateway_ipv4: ipv4(
                "RUSTERON_DPDK_SENDER_GATEWAY",
                &required("RUSTERON_DPDK_SENDER_GATEWAY")?,
            )?,
        },
        receiver: DpdkPortConfig {
            pci_address: required("RUSTERON_DPDK_RECEIVER_PCI")?,
            local_ipv4: receiver_addr,
            prefix_len: receiver_prefix,
            gateway_ipv4: ipv4(
                "RUSTERON_DPDK_RECEIVER_GATEWAY",
                &required("RUSTERON_DPDK_RECEIVER_GATEWAY")?,
            )?,
        },
        file_prefix: required("RUSTERON_DPDK_FILE_PREFIX")?,
        test_vdev, // production selectors are PCI-only unless the test lever is set (plan §11.2)
        hugepage_dir: PathBuf::from(env::var("RUSTERON_DPDK_HUGE_DIR").unwrap_or_else(|_| "/dev/hugepages".into())),
        rx_descriptors: num_or("RUSTERON_DPDK_RX_DESCRIPTORS", 1024)?,
        tx_descriptors: num_or("RUSTERON_DPDK_TX_DESCRIPTORS", 1024)?,
        mbufs_per_port: num_or("RUSTERON_DPDK_MBUFS_PER_PORT", 65536)?,
        mempool_cache: num_or("RUSTERON_DPDK_MEMPOOL_CACHE", 256)?,
        burst_size: num_or("RUSTERON_DPDK_BURST_SIZE", 32)?,
        max_aeron_mtu: num_or("RUSTERON_DPDK_MAX_AERON_MTU", 1408)?,
    })
}

fn required(name: &str) -> Result<String, DpdkError> {
    env::var(name).map_err(|_| DpdkError::MissingEnvironment(name.to_string()))
}

fn ipv4(name: &str, value: &str) -> Result<Ipv4Addr, DpdkError> {
    value
        .parse()
        .map_err(|_| DpdkError::InvalidEnvironment(format!("{name}={value:?} is not a valid IPv4 address")))
}

/// Parse `address/prefix` into the address and prefix length.
fn parse_cidr(role: &str, value: &str) -> Result<(Ipv4Addr, u8), DpdkError> {
    let var = format!("RUSTERON_DPDK_{}_IPV4_CIDR", role.to_uppercase());
    let (ip, prefix) = value.split_once('/').ok_or_else(|| {
        DpdkError::InvalidEnvironment(format!("{var}={value:?} must be address/prefix (e.g. 10.0.0.1/24)"))
    })?;
    let addr = ipv4(&var, ip)?;
    let prefix_len: u8 = prefix
        .trim()
        .parse()
        .map_err(|_| DpdkError::InvalidEnvironment(format!("{var} prefix {prefix:?} is not a number")))?;
    if !(1..=32).contains(&prefix_len) {
        return Err(DpdkError::InvalidEnvironment(format!(
            "{var} prefix {prefix_len} must be in 1..=32"
        )));
    }
    Ok((addr, prefix_len))
}

/// Parse a numeric environment value, using `default` when absent.
fn num_or<T: FromStr>(name: &str, default: T) -> Result<T, DpdkError>
where
    T::Err: std::fmt::Display,
{
    match env::var(name) {
        Err(_) => Ok(default),
        Ok(v) => v
            .trim()
            .parse()
            .map_err(|_| DpdkError::InvalidEnvironment(format!("{name}={v:?} is not a valid number"))),
    }
}
