//! Kubelet device-plugin v1beta1 service (plan §10.3). Advertises one device
//! per healthy VFIO ENA pair under `rusteron.io/dpdk-ena-pair`; `Allocate`
//! atomically injects the pair's sender/receiver DPDK env and the VFIO device
//! nodes into the media driver pod.

use crate::inventory::{Inventory, Pair, Port};
use crate::{
    device_plugin_server, registration_server, AllocateRequest, AllocateResponse, ContainerAllocateResponse, Device,
    DevicePluginOptions, DeviceSpec, Empty, ListAndWatchResponse, NumaNode, PreStartContainerRequest,
    PreStartContainerResponse, PreferredAllocationRequest, PreferredAllocationResponse, RegisterRequest, TopologyInfo,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

/// Kubelet accepts a Register call and the plugin then serves the device API.
/// We hold no state; a 200 is all the kubelet expects (plan §10.3).
#[derive(Clone, Default)]
pub struct RegistrationService;

#[tonic::async_trait]
impl registration_server::Registration for RegistrationService {
    async fn register(&self, _request: Request<RegisterRequest>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }
}

/// Host paths the live health probe inspects. Both are overridable so tests
/// can point at fixtures.
#[derive(Clone, Debug)]
pub struct SysfsRoots {
    pub sys_root: PathBuf,
    pub vfio_root: PathBuf,
}

/// A pair is healthy only while both BDFs stay bound to vfio-pci and their
/// IOMMU group device nodes still exist (plan §10.3).
pub type HealthProbe = fn(&SysfsRoots, &Pair) -> bool;

pub fn sysfs_health(roots: &SysfsRoots, pair: &Pair) -> bool {
    [&pair.sender, &pair.receiver].iter().all(|p| {
        let bound = roots
            .sys_root
            .join("bus/pci/devices")
            .join(&p.pci)
            .join("driver")
            .read_link()
            .ok()
            .map(|t| t.to_string_lossy().contains("vfio-pci"))
            .unwrap_or(false);
        bound && roots.vfio_root.join(&p.iommu_group).exists()
    })
}

/// DevicePlugin service implementation.
#[derive(Clone)]
pub struct DevicePluginService {
    inventory: Arc<Inventory>,
    roots: SysfsRoots,
    health: HealthProbe,
    poll_interval: Duration,
}

impl DevicePluginService {
    pub fn new(inventory: Inventory, roots: SysfsRoots, health: HealthProbe, poll_interval: Duration) -> Self {
        Self {
            inventory: Arc::new(inventory),
            roots,
            health,
            poll_interval,
        }
    }

    /// Current healthy devices: one per pair that passes the live probe.
    fn healthy_devices(&self) -> Vec<Device> {
        self.inventory
            .pairs
            .iter()
            .filter(|p| (self.health)(&self.roots, p))
            .map(|p| Device {
                id: p.id.clone(),
                health: "Healthy".into(),
                topology: (p.sender.numa_node >= 0).then(|| TopologyInfo {
                    nodes: vec![NumaNode {
                        id: p.sender.numa_node as i64,
                    }],
                }),
            })
            .collect()
    }

    fn allocate_container(&self, devices_ids: &[String]) -> Result<ContainerAllocateResponse, Status> {
        let id = devices_ids
            .first()
            .ok_or_else(|| Status::invalid_argument("no device requested"))?;
        let pair = self
            .inventory
            .pairs
            .iter()
            .find(|p| &p.id == id)
            .ok_or_else(|| Status::not_found(format!("unknown device {id}")))?;
        if !(self.health)(&self.roots, pair) {
            return Err(Status::failed_precondition(format!("device {id} is not healthy")));
        }
        Ok(allocation_for(pair))
    }
}

#[tonic::async_trait]
impl device_plugin_server::DevicePlugin for DevicePluginService {
    type ListAndWatchStream = Pin<Box<dyn tokio_stream::Stream<Item = Result<ListAndWatchResponse, Status>> + Send>>;

    async fn get_device_plugin_options(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<DevicePluginOptions>, Status> {
        // No PreStartContainer, no preferred-allocation hinting.
        Ok(Response::new(DevicePluginOptions {
            pre_start_required: false,
            get_preferred_allocation_available: false,
        }))
    }

    async fn list_and_watch(&self, _request: Request<Empty>) -> Result<Response<Self::ListAndWatchStream>, Status> {
        let (tx, rx) = mpsc::channel::<Result<ListAndWatchResponse, Status>>(8);
        let this = self.clone();
        tokio::spawn(async move {
            let mut last: Option<Vec<Device>> = None;
            loop {
                let devices = this.healthy_devices();
                if last.as_ref() != Some(&devices) {
                    let next = ListAndWatchResponse {
                        devices: devices.clone(),
                    };
                    if tx.send(Ok(next)).await.is_err() {
                        break; // kubelet disconnected
                    }
                    last = Some(devices);
                }
                tokio::time::sleep(this.poll_interval).await;
            }
        });
        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    async fn get_preferred_allocation(
        &self,
        _request: Request<PreferredAllocationRequest>,
    ) -> Result<Response<PreferredAllocationResponse>, Status> {
        // Not advertised (get_preferred_allocation_available = false); the
        // kubelet falls back to round-robin. Empty per the v1beta1 contract.
        Ok(Response::new(PreferredAllocationResponse {
            container_responses: vec![],
        }))
    }

    async fn allocate(&self, request: Request<AllocateRequest>) -> Result<Response<AllocateResponse>, Status> {
        let mut container_responses = Vec::new();
        for cr in &request.into_inner().container_requests {
            container_responses.push(self.allocate_container(&cr.devices_ids)?);
        }
        Ok(Response::new(AllocateResponse { container_responses }))
    }

    async fn pre_start_container(
        &self,
        _request: Request<PreStartContainerRequest>,
    ) -> Result<Response<PreStartContainerResponse>, Status> {
        Ok(Response::new(PreStartContainerResponse {}))
    }
}

/// Build the Allocate response for one pair: both roles' DPDK env plus the
/// VFIO control device and each distinct IOMMU group device (plan §10.3).
fn allocation_for(pair: &Pair) -> ContainerAllocateResponse {
    let mut envs = HashMap::new();
    envs.insert("RUSTERON_MEDIA_DRIVER_TRANSPORT".to_string(), "dpdk-ena".to_string());
    envs.insert(
        "RUSTERON_DPDK_FILE_PREFIX".to_string(),
        format!("rusteron-dpdk-{}", pair.id),
    );
    put_role(&mut envs, "SENDER", &pair.sender);
    put_role(&mut envs, "RECEIVER", &pair.receiver);

    let mut devices = vec![device_spec("/dev/vfio/vfio")];
    let mut groups = HashSet::new();
    groups.insert(pair.sender.iommu_group.as_str());
    groups.insert(pair.receiver.iommu_group.as_str());
    for group in groups {
        devices.push(device_spec(&format!("/dev/vfio/{group}")));
    }

    ContainerAllocateResponse {
        envs,
        mounts: vec![],
        devices,
        annotations: HashMap::new(),
        cdi_devices: vec![],
    }
}

// ponytail: role builds RUSTERON_DPDK_{SENDER|RECEIVER}_{PCI|IPV4_CIDR|GATEWAY}
// and must match the env.rs required() names; pinned by the
// allocation_injects_pair_env_and_vfio_nodes test.
fn put_role(envs: &mut HashMap<String, String>, role: &str, port: &Port) {
    let p = format!("RUSTERON_DPDK_{role}");
    envs.insert(format!("{p}_PCI"), port.pci.clone());
    envs.insert(format!("{p}_IPV4_CIDR"), cidr(port));
    envs.insert(format!("{p}_GATEWAY"), port.gateway.clone());
}

/// `address/prefix`, the format the DPDK env contract parses
/// (rusteron-media-driver/src/dpdk/env.rs parse_cidr).
fn cidr(port: &Port) -> String {
    format!("{}/{}", port.ipv4, port.prefix_len)
}

fn device_spec(path: &str) -> DeviceSpec {
    DeviceSpec {
        container_path: path.into(),
        host_path: path.into(),
        permissions: "rwm".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::fixtures::fixture;

    fn pair() -> Pair {
        fixture().pairs.remove(0)
    }

    fn service(health: HealthProbe) -> DevicePluginService {
        DevicePluginService::new(
            fixture(),
            SysfsRoots {
                sys_root: PathBuf::from("/sys"),
                vfio_root: PathBuf::from("/dev/vfio"),
            },
            health,
            Duration::from_secs(5),
        )
    }

    fn always_healthy(_: &SysfsRoots, _: &Pair) -> bool {
        true
    }

    fn never_healthy(_: &SysfsRoots, _: &Pair) -> bool {
        false
    }

    #[test]
    fn allocation_injects_pair_env_and_vfio_nodes() {
        let resp = allocation_for(&pair());
        assert_eq!(resp.envs["RUSTERON_MEDIA_DRIVER_TRANSPORT"], "dpdk-ena");
        assert_eq!(resp.envs["RUSTERON_DPDK_SENDER_PCI"], "0000:00:06.0");
        assert_eq!(resp.envs["RUSTERON_DPDK_SENDER_IPV4_CIDR"], "10.0.0.1/24");
        assert_eq!(resp.envs["RUSTERON_DPDK_SENDER_GATEWAY"], "10.0.0.1");
        assert_eq!(resp.envs["RUSTERON_DPDK_RECEIVER_PCI"], "0000:00:07.0");
        assert_eq!(resp.envs["RUSTERON_DPDK_RECEIVER_IPV4_CIDR"], "10.0.0.1/24");
        let paths: Vec<&str> = resp.devices.iter().map(|d| d.host_path.as_str()).collect();
        assert!(paths.contains(&"/dev/vfio/vfio"));
        assert!(paths.contains(&"/dev/vfio/9"));
        assert!(paths.contains(&"/dev/vfio/10"));
        assert!(resp.devices.iter().all(|d| d.permissions == "rwm"));
    }

    #[test]
    fn shared_iommu_group_is_deduplicated() {
        let mut p = pair();
        p.receiver.iommu_group = p.sender.iommu_group.clone();
        let resp = allocation_for(&p);
        // /dev/vfio/vfio + one group device.
        assert_eq!(resp.devices.len(), 2);
    }

    #[test]
    fn only_healthy_pairs_are_advertised() {
        assert_eq!(service(always_healthy).healthy_devices().len(), 1);
        assert!(service(never_healthy).healthy_devices().is_empty());
        let dev = service(always_healthy).healthy_devices().remove(0);
        assert_eq!(dev.id, "dpdk-pair-0");
        assert_eq!(dev.health, "Healthy");
    }

    #[test]
    fn allocate_rejects_unknown_and_unhealthy() {
        let svc = service(always_healthy);
        assert!(svc.allocate_container(&["nope".into()]).is_err());
        let bad = service(never_healthy);
        assert!(bad.allocate_container(&["dpdk-pair-0".into()]).is_err());
        let ok = svc.allocate_container(&["dpdk-pair-0".into()]).unwrap();
        assert!(!ok.envs.is_empty());
    }
}
