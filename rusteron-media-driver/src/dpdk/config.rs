//! Typed DPDK transport configuration and validation (plan §5, §6.5).

use crate::dpdk::error::DpdkError;
use std::net::Ipv4Addr;
use std::path::PathBuf;

/// Per-port DPDK configuration for one ENA device.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DpdkPortConfig {
    /// Canonical `dddd:bb:ss.f` PCI BDF of the ENA device.
    pub pci_address: String,
    /// IPv4 address assigned to this ENA on its subnet.
    pub local_ipv4: Ipv4Addr,
    /// Subnet prefix length (`1..=32`).
    pub prefix_len: u8,
    /// VPC router address for this subnet.
    pub gateway_ipv4: Ipv4Addr,
}

/// Validated transport configuration covering both ENA roles and the EAL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DpdkTransportConfig {
    pub sender: DpdkPortConfig,
    pub receiver: DpdkPortConfig,
    /// DPDK EAL file prefix, unique per process (`[A-Za-z0-9_-]{1,64}`).
    pub file_prefix: String,
    /// Accept DPDK virtual-device names (e.g. `net_tap0`) instead of PCI BDFs.
    ///
    /// Test-only (plan §11.2): the environment parser never sets this, so
    /// production configurations stay PCI-only. When set, the two selectors
    /// are each validated as a canonical PCI BDF *or* a vdev name, and the
    /// native EAL emits `--vdev=<name>` instead of `-a <bdf>`.
    pub test_vdev: bool,
    /// Hugepage mount. Must be absolute; hugetlbfs backing is verified natively
    /// at EAL initialization (plan §6.5, Ticket 3).
    pub hugepage_dir: PathBuf,
    pub rx_descriptors: u16,
    pub tx_descriptors: u16,
    pub mbufs_per_port: u32,
    pub mempool_cache: u16,
    pub burst_size: u16,
    pub max_aeron_mtu: usize,
}

impl DpdkTransportConfig {
    /// Validate every cross-field invariant in plan §6.5.
    ///
    /// Returns [`DpdkError::InvalidConfiguration`] with the failing field (and
    /// port role where applicable) named in the message.
    pub fn validate(&self) -> Result<(), DpdkError> {
        validate_port("sender", &self.sender, self.test_vdev)?;
        validate_port("receiver", &self.receiver, self.test_vdev)?;

        if self.sender.pci_address == self.receiver.pci_address {
            return Err(DpdkError::InvalidConfiguration(format!(
                "sender.pci_address and receiver.pci_address must differ (both are {})",
                self.sender.pci_address
            )));
        }

        if !valid_file_prefix(&self.file_prefix) {
            return Err(DpdkError::InvalidConfiguration(format!(
                "file_prefix {:?} must match [A-Za-z0-9_-]{{1,64}}",
                self.file_prefix
            )));
        }

        if !self.hugepage_dir.is_absolute() {
            return Err(DpdkError::InvalidConfiguration(format!(
                "hugepage_dir {:?} must be an absolute path to the hugetlbfs mount",
                self.hugepage_dir
            )));
        }

        if !(64..=8192).contains(&self.rx_descriptors) {
            return Err(DpdkError::InvalidConfiguration(format!(
                "rx_descriptors {} must be in 64..=8192",
                self.rx_descriptors
            )));
        }
        if !(64..=8192).contains(&self.tx_descriptors) {
            return Err(DpdkError::InvalidConfiguration(format!(
                "tx_descriptors {} must be in 64..=8192",
                self.tx_descriptors
            )));
        }
        if !(1..=256).contains(&self.burst_size) {
            return Err(DpdkError::InvalidConfiguration(format!(
                "burst_size {} must be in 1..=256",
                self.burst_size
            )));
        }

        // The mbuf pool must cover both descriptor rings plus four bursts of
        // headroom (plan §6.5).
        let required = u64::from(self.rx_descriptors) + u64::from(self.tx_descriptors) + 4 * u64::from(self.burst_size);
        if u64::from(self.mbufs_per_port) < required {
            return Err(DpdkError::InvalidConfiguration(format!(
                "mbufs_per_port {} is below the required {required} (rx + tx descriptors + 4 bursts)",
                self.mbufs_per_port
            )));
        }

        if self.max_aeron_mtu % 32 != 0 || self.max_aeron_mtu > 1472 {
            return Err(DpdkError::InvalidConfiguration(format!(
                "max_aeron_mtu {} must be 32-aligned and at most 1472",
                self.max_aeron_mtu
            )));
        }

        Ok(())
    }
}

fn validate_port(role: &str, port: &DpdkPortConfig, test_vdev: bool) -> Result<(), DpdkError> {
    if !valid_pci(&port.pci_address) && !(test_vdev && valid_vdev(&port.pci_address)) {
        return Err(DpdkError::InvalidConfiguration(format!(
            "{role}.pci_address {:?} is not canonical dddd:bb:ss.f{}",
            port.pci_address,
            if test_vdev { " nor a DPDK vdev name" } else { "" }
        )));
    }
    if !is_unicast(port.local_ipv4) {
        return Err(DpdkError::InvalidConfiguration(format!(
            "{role}.local_ipv4 {ip} is not a unicast IPv4 address",
            ip = port.local_ipv4
        )));
    }
    if !is_unicast(port.gateway_ipv4) {
        return Err(DpdkError::InvalidConfiguration(format!(
            "{role}.gateway_ipv4 {ip} is not a unicast IPv4 address",
            ip = port.gateway_ipv4
        )));
    }
    if !(1..=32).contains(&port.prefix_len) {
        return Err(DpdkError::InvalidConfiguration(format!(
            "{role}.prefix_len {} must be in 1..=32",
            port.prefix_len
        )));
    }

    // Gateway must be inside the configured local subnet (plan §6.5).
    // `u32::from(Ipv4Addr)` keeps the first octet in the high byte, so the
    // prefix mask must set the *network* bits, i.e. `u32::MAX << (32 - len)`.
    let shift = 32 - u32::from(port.prefix_len);
    let mask = u32::MAX << shift;
    if u32::from(port.local_ipv4) & mask != u32::from(port.gateway_ipv4) & mask {
        return Err(DpdkError::InvalidConfiguration(format!(
            "{role}.gateway_ipv4 {gw} is not in the {ip}/{prefix} subnet of {role}.local_ipv4",
            gw = port.gateway_ipv4,
            ip = port.local_ipv4,
            prefix = port.prefix_len,
        )));
    }

    Ok(())
}

/// `std`'s `Ipv4Addr` has no stable `is_unicast`, so spell it out: not
/// multicast, not the limited broadcast, not `0.0.0.0`.
fn is_unicast(addr: Ipv4Addr) -> bool {
    !addr.is_multicast() && !addr.is_broadcast() && !addr.is_unspecified()
}

/// Canonical PCI BDF syntax `dddd:bb:ss.f` (4-2-2 hex, dot, 1 hex).
fn valid_pci(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 12 {
        return false;
    }
    let hex = |i: usize| b[i].is_ascii_hexdigit();
    hex(0)
        && hex(1)
        && hex(2)
        && hex(3)
        && b[4] == b':'
        && hex(5)
        && hex(6)
        && b[7] == b':'
        && hex(8)
        && hex(9)
        && b[10] == b'.'
        && hex(11)
}

/// DPDK virtual-device name (e.g. `net_tap0`, `net_pcap0`), test-only.
///
/// Must fit the 16-byte native selector buffer and must not contain `:` so the
/// native EAL can distinguish a vdev name from a PCI BDF by shape.
fn valid_vdev(s: &str) -> bool {
    let n = s.len();
    n >= 1 && n <= 15 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn valid_file_prefix(s: &str) -> bool {
    let n = s.len();
    n >= 1 && n <= 64 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
