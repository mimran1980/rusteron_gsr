# AWS EKS DPDK ENA deployment (plan §10)

Deployment assets for running the `rusteron-media-driver` DPDK ENA
kernel-bypass transport on Amazon EKS (Amazon Linux 2023, Linux x86_64 Nitro).
One Media Driver pod per dedicated node, with the node's primary ENA left
kernel-owned for EKS/DNS/telemetry and used as the kernel-UDP rollback path.

```
deploy/aws-eks/
├── Dockerfile.media-driver      # AL2023 runtime image, builds media_driver with static,dpdk
├── entrypoint.sh                # container entrypoint: cpuset -> AERON CPU affinities
└── node/
    ├── bootstrap-dpdk-ena.sh    # reboot-safe: discover/bind secondaries, write pair inventory
    ├── preflight.sh             # IOMMU/VFIO/hugepages/NUMA/ENA identity checks
    ├── rusteron-dpdk-ena.service# systemd oneshot: run bootstrap before kubelet
    └── test/
        └── test-bootstrap.sh    # fixture tests for the node scripts
```

## 1. Runtime image

Build from the repo root (the `Dockerfile.media-driver` build stage compiles
the workspace with `--features static,dpdk` on AL2023):

```bash
docker build -f deploy/aws-eks/Dockerfile.media-driver -t rusteron-media-driver:dpdk .
docker run --rm rusteron-media-driver:dpdk ldd /usr/local/bin/media_driver
```

The image records the exact DPDK/ENA/kernel/Aeron versions it was built
against in `/etc/rusteron-dpdk-versions.txt`. The container itself only needs
`/dev/hugepages`; the vfio-bound ENAs and the hugepage reservation live on the
node and are mounted in by the kubelet (see below).

> The image is `--platform=linux/amd64` (EKS Nitro is x86_64; the `dpdk`
> feature rejects any other target at build time).

## 2. Node bootstrap

Run on each dedicated EKS worker node before the Media Driver workload is
scheduled. It never touches the primary (default-route) ENA.

```bash
sudo cp node/bootstrap-dpdk-ena.sh node/preflight.sh /usr/local/sbin/
sudo cp node/rusteron-dpdk-ena.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now rusteron-dpdk-ena.service
```

What it does:

1. **Preflight** (`preflight.sh`) — fails the node loudly if IOMMU groups are
   absent, `vfio-pci`/`vfio_iommu_type1` are missing (unsafe no-IOMMU mode),
   no 2 MiB hugepages are reserved at `/dev/hugepages`, NUMA topology is
   absent, fewer than three ENAs are discoverable, or a secondary ENA lacks a
   complete IMDS identity.
2. **Discovery** — reads the instance's ENI list from IMDS; the primary ENA
   (default-route, or IMDS `device-number` 0) is preserved and excluded. The
   first two secondary ENAs become the Aeron sender/receiver pair.
3. **Bind** — unbinds each secondary ENA from the kernel `ena` driver and
   rebinds it to `vfio-pci` via `driver_override`.
4. **Inventory** — writes `/var/lib/rusteron-dpdk/ena-pairs.json` atomically
   (temp file + `mv`), recording per role: PCI BDF, VFIO IOMMU group, ENI id,
   MAC, IPv4, prefix length, subnet CIDR, gateway, NUMA node, and health.

The systemd unit (`Type=oneshot`, `RemainAfterExit=yes`) re-runs the bootstrap
at every boot, so a reboot restores the VFIO bindings and regenerates a fresh
inventory before kubelet schedules pods. A bootstrap failure leaves the node
usable for kernel-UDP workloads (the failure is visible via
`systemctl status rusteron-dpdk-ena`); the device plugin simply advertises no
DPDK pair.

### Node requirements (plan §10.2)

- AL2023 x86_64 Nitro instance with ≥ 3 ENAs (primary + sender + receiver).
- IOMMU enabled (`iommu=pt`); `vfio-pci` and `vfio_iommu_type1` loaded.
- 2 MiB hugepages reserved (`sysctl vm.nr_hugepages`) and mounted:
  `mount -t hugetlbfs -o pagesize=2M hugetlbfs /dev/hugepages`.
- Secondary ENAs created by the node-group launch template, tagged
  `node.k8s.amazonaws.com/no_manage=true` so AWS VPC CNI does not claim them.
- CPU Manager `static` and Topology Manager `single-numa-node`; system/Kubernetes
  CPUs reserved away from the DPDK pod's exclusive CPU set.

## 3. Entrypoint

`entrypoint.sh` reads the pod's effective cpuset
(`/sys/fs/cgroup/cpuset.cpus.effective`), requires at least three exclusive
CPUs, and assigns the first three to the Aeron conductor, sender, and receiver
threads via `AERON_CONDUCTOR/SENDER/RECEIVER_CPU_AFFINITY`. It sets the
remaining plan §6.4 driver settings (DEDICATED threading, spin idle, disjoint
wildcard port ranges, `AERON_MTU_LENGTH=1408`) and
`RUSTERON_MEDIA_DRIVER_TRANSPORT=dpdk-ena`, then `exec`s the media driver.
Any setting already present in the environment wins (the caller can override).

The `RUSTERON_DPDK_SENDER/RECEIVER_{PCI,IPV4_CIDR,GATEWAY}` values are injected
by the device plugin at allocation time (Ticket 10); the media driver fails
fast and exits nonzero if any are missing — it never falls back to the socket
driver.

## 4. Fixture tests

The node scripts are tested against fake sysfs/IMDS trees before any live
node (plan §9):

```bash
bash node/test/test-bootstrap.sh
```

The test builds three fake Amazon ENAs, asserts preflight passes, that the
bootstrap writes a complete inventory (primary preserved, both roles with PCI,
IOMMU group, ENI, IPv4/prefix/subnet/gateway, NUMA, health), and that removing
`vfio_iommu_type1` makes preflight fail. The bootstrap runs with
`RUSTERON_DRY_RUN=1`, so no host sysfs is ever touched.

## 5. Next: EKS scheduling (Ticket 10)

The device plugin (`rusteron-dpdk-device-plugin`) reads
`/var/lib/rusteron-dpdk/ena-pairs.json`, advertises
`rusteron.io/dpdk-ena-pair`, and injects the per-role DPDK variables and VFIO
device nodes into Media Driver pods. See that ticket's deployment manifests
(`kustomization.yaml`, daemonsets) once they land.
