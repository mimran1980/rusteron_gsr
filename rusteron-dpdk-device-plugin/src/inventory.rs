//! Pair inventory (`/var/lib/rusteron-dpdk/ena-pairs.json`, plan §10.2): the
//! exact shape written by `deploy/aws-eks/node/bootstrap-dpdk-ena.sh`, plus
//! strict validation.
//!
//! The plugin refuses any inventory with duplicate BDFs, duplicate IOMMU
//! groups, a member MAC equal to the primary ENA, or incomplete network
//! identity (plan §10.3) — it must never advertise or allocate an unsafe pair.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::path::Path;

/// One node's pair inventory.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Inventory {
    #[serde(default)]
    pub generated_at: String,
    pub primary_ena_mac: String,
    pub pairs: Vec<Pair>,
}

/// One sender/receiver ENA pair.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pair {
    pub id: String,
    pub sender: Port,
    pub receiver: Port,
}

/// Network identity of one ENA role. Missing identity fields default so that
/// validation can reject them with a precise reason instead of a parse error.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Port {
    pub pci: String,
    pub iommu_group: String,
    pub eni_id: String,
    pub mac: String,
    pub ipv4: String,
    #[serde(default)]
    pub prefix_len: u8,
    #[serde(default)]
    pub subnet_cidr: String,
    #[serde(default)]
    pub gateway: String,
    #[serde(default = "default_numa")]
    pub numa_node: i32,
    #[serde(default = "default_health")]
    pub health: String,
}

fn default_numa() -> i32 {
    -1
}

fn default_health() -> String {
    "healthy".into()
}

/// Why an inventory was rejected. Kubelet must never see a pair the plugin
/// cannot allocate safely.
#[derive(Debug)]
pub enum InventoryError {
    /// File could not be read.
    Io(String, std::io::Error),
    /// JSON did not parse.
    Parse(String),
    /// Validation rule violated (message is the reason).
    Validation(String),
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InventoryError::Io(path, e) => write!(f, "cannot read {path}: {e}"),
            InventoryError::Parse(e) => write!(f, "malformed inventory JSON: {e}"),
            InventoryError::Validation(msg) => write!(f, "unsafe inventory: {msg}"),
        }
    }
}

impl std::error::Error for InventoryError {}

impl Inventory {
    /// Load and validate the inventory at `path`.
    pub fn load(path: &Path) -> Result<Inventory, InventoryError> {
        let raw = std::fs::read_to_string(path).map_err(|e| InventoryError::Io(path.display().to_string(), e))?;
        let inv: Inventory = serde_json::from_str(&raw).map_err(|e| InventoryError::Parse(e.to_string()))?;
        inv.validate()?;
        Ok(inv)
    }

    /// Reject duplicate/unsafe inventories (plan §10.3).
    pub fn validate(&self) -> Result<(), InventoryError> {
        if self.primary_ena_mac.trim().is_empty() {
            return Err(InventoryError::Validation("missing primary_ena_mac".into()));
        }
        if self.pairs.is_empty() {
            return Err(InventoryError::Validation("no ENA pairs".into()));
        }
        let primary = self.primary_ena_mac.trim().to_ascii_lowercase();
        let mut ids = HashSet::new();
        let mut bdfs = HashSet::new();
        let mut groups = HashSet::new();
        for (i, pair) in self.pairs.iter().enumerate() {
            if !ids.insert(pair.id.as_str()) {
                return Err(InventoryError::Validation(format!("duplicate pair id {:?}", pair.id)));
            }
            for (role, port) in [("sender", &pair.sender), ("receiver", &pair.receiver)] {
                validate_port(port)
                    .map_err(|reason| InventoryError::Validation(format!("pair {i} {role}: {reason}")))?;
                if !bdfs.insert(port.pci.as_str()) {
                    return Err(InventoryError::Validation(format!(
                        "duplicate BDF {} in pair {i} {role}",
                        port.pci
                    )));
                }
                if !groups.insert(port.iommu_group.as_str()) {
                    return Err(InventoryError::Validation(format!(
                        "duplicate IOMMU group {} in pair {i} {role}",
                        port.iommu_group
                    )));
                }
                if port.mac.trim().to_ascii_lowercase() == primary {
                    return Err(InventoryError::Validation(format!(
                        "pair {i} {role} {} is the primary ENA",
                        port.pci
                    )));
                }
            }
        }
        Ok(())
    }
}

fn validate_port(port: &Port) -> Result<(), String> {
    if port.pci.trim().is_empty() {
        return Err("empty pci".into());
    }
    if !is_bdf(&port.pci) {
        return Err(format!("pci {:?} is not a BDF (DDDD:BB:DD.F)", port.pci));
    }
    if port.iommu_group.trim().is_empty() {
        return Err("empty iommu_group".into());
    }
    if port.eni_id.trim().is_empty() {
        return Err("empty eni_id".into());
    }
    if port.mac.trim().is_empty() {
        return Err("empty mac".into());
    }
    if port.ipv4.trim().is_empty() {
        return Err("empty ipv4".into());
    }
    if port.subnet_cidr.trim().is_empty() {
        return Err("empty subnet_cidr".into());
    }
    if port.gateway.trim().is_empty() {
        return Err("empty gateway".into());
    }
    if !(1..=32).contains(&port.prefix_len) {
        return Err(format!("prefix_len {} not in 1..=32", port.prefix_len));
    }
    if port.numa_node < -1 {
        return Err(format!("numa_node {} < -1", port.numa_node));
    }
    Ok(())
}

/// Loose check for the `DDDD:BB:DD.F` BDF layout the node bootstrap writes
/// (`basename` of a `/sys/bus/pci/devices/*` directory, e.g. `0000:00:06.0`).
fn is_bdf(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 12
        && b[4] == b':'
        && b[7] == b':'
        && b[10] == b'.'
        && b[..4]
            .iter()
            .chain(&b[5..7])
            .chain(&b[8..10])
            .all(u8::is_ascii_hexdigit)
        && b[11].is_ascii_digit()
}

/// Shared test fixtures (used by the inventory and service test suites).
#[cfg(test)]
pub(crate) mod fixtures {
    use super::{Inventory, Pair, Port};

    pub(crate) fn port(pci: &str, group: &str, mac: &str) -> Port {
        Port {
            pci: pci.into(),
            iommu_group: group.into(),
            eni_id: "eni-0123456789abcdef0".into(),
            mac: mac.into(),
            ipv4: "10.0.0.1".into(),
            prefix_len: 24,
            subnet_cidr: "10.0.0.0/24".into(),
            gateway: "10.0.0.1".into(),
            numa_node: 0,
            health: "healthy".into(),
        }
    }

    pub(crate) fn fixture() -> Inventory {
        Inventory {
            generated_at: "2026-08-07T00:00:00Z".into(),
            primary_ena_mac: "0a:00:00:00:00:01".into(),
            pairs: vec![Pair {
                id: "dpdk-pair-0".into(),
                sender: port("0000:00:06.0", "9", "0a:00:00:00:00:02"),
                receiver: port("0000:00:07.0", "10", "0a:00:00:00:00:03"),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;

    #[test]
    fn accepts_valid_inventory() {
        fixture().validate().unwrap();
    }

    #[test]
    fn accepts_fixture_json_from_bootstrap_shape() {
        let json = r#"{
          "generated_at": "2026-08-07T00:00:00Z",
          "primary_ena_mac": "0a:00:00:00:00:01",
          "pairs": [{
            "id": "dpdk-pair-0",
            "sender":  {"pci":"0000:00:06.0","iommu_group":"9","eni_id":"eni-a","mac":"0a:00:00:00:00:02","ipv4":"10.0.0.1","prefix_len":24,"subnet_cidr":"10.0.0.0/24","gateway":"10.0.0.1","numa_node":0,"health":"healthy"},
            "receiver":{"pci":"0000:00:07.0","iommu_group":"10","eni_id":"eni-b","mac":"0a:00:00:00:00:03","ipv4":"10.0.0.2","prefix_len":24,"subnet_cidr":"10.0.0.0/24","gateway":"10.0.0.1","numa_node":0,"health":"healthy"}
          }]
        }"#;
        Inventory::validate(&serde_json::from_str(json).unwrap()).unwrap();
    }

    #[test]
    fn rejects_duplicate_bdf() {
        let mut inv = fixture();
        inv.pairs[0].receiver.pci = "0000:00:06.0".into();
        let err = inv.validate().unwrap_err().to_string();
        assert!(err.contains("duplicate BDF"), "{err}");
    }

    #[test]
    fn rejects_duplicate_group() {
        let mut inv = fixture();
        inv.pairs[0].receiver.iommu_group = "9".into();
        let err = inv.validate().unwrap_err().to_string();
        assert!(err.contains("duplicate IOMMU group"), "{err}");
    }

    #[test]
    fn rejects_primary_ena_in_pair() {
        let mut inv = fixture();
        inv.pairs[0].sender.mac = inv.primary_ena_mac.clone();
        let err = inv.validate().unwrap_err().to_string();
        assert!(err.contains("primary ENA"), "{err}");
    }

    #[test]
    fn rejects_incomplete_identity() {
        let mut inv = fixture();
        inv.pairs[0].sender.gateway.clear();
        let err = inv.validate().unwrap_err().to_string();
        assert!(err.contains("empty gateway"), "{err}");
    }

    #[test]
    fn rejects_bad_bdf_and_prefix() {
        let mut inv = fixture();
        inv.pairs[0].sender.pci = "00:06.0".into();
        assert!(inv.validate().is_err());
        inv = fixture();
        inv.pairs[0].receiver.prefix_len = 33;
        assert!(inv.validate().is_err());
    }

    #[test]
    fn rejects_empty_pairs_and_duplicate_ids() {
        let mut inv = fixture();
        inv.pairs.clear();
        assert!(inv.validate().is_err());
        inv = fixture();
        inv.pairs.push(inv.pairs[0].clone());
        let err = inv.validate().unwrap_err().to_string();
        assert!(err.contains("duplicate pair id"), "{err}");
    }
}
