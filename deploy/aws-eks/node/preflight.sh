#!/usr/bin/env bash
# Node preflight for the rusteron-media-driver DPDK ENA workload (plan §9,
# §10.2). Verifies the host is safe to hand two secondary ENAs to DPDK:
#
#   * IOMMU is enabled with real groups (no unsafe no-IOMMU vfio mode).
#   * vfio-pci and vfio_iommu_type1 are available.
#   * 2 MiB hugepages are reserved and mounted at /dev/hugepages.
#   * NUMA topology is present (hugepage/node locality matters on Nitro).
#   * At least the primary ENA + two secondary ENAs are discoverable.
#   * The default-route (primary) ENA is NOT among the DPDK candidates.
#   * IMDS returns a parseable IP/gateway identity for each candidate.
#
# Testable against a fake fixture root: set RUSTERON_SYSFS_ROOT and
# RUSTERON_IMDS_ROOT to the fixture paths (see node/test/test-bootstrap.sh).
#
# Exit 0 = preflight OK. Exit 1 = unsafe/incomplete, with the failing check
# named. Accepts no arguments.
set -euo pipefail

RUSTERON_SYSFS_ROOT="${RUSTERON_SYSFS_ROOT:-/sys}"
RUSTERON_IMDS_ROOT="${RUSTERON_IMDS_ROOT:-http://169.254.169.254/latest/meta-data}"
RUSTERON_HUGEPAGES_DIR="${RUSTERON_HUGEPAGES_DIR:-/dev/hugepages}"
# Fixture-test overrides: a known primary MAC and a non-mount hugepage root
# (see node/test/test-bootstrap.sh). Absent in production.
RUSTERON_PRIMARY_MAC="${RUSTERON_PRIMARY_MAC:-}"
RUSTERON_SKIP_MOUNT_CHECK="${RUSTERON_SKIP_MOUNT_CHECK:-0}"

fail() { echo "preflight FAIL: $*" >&2; exit 1; }
ok()   { echo "preflight ok: $*"; }

# IMDSv2-aware fetch; a non-http root is treated as a local fixture path.
imds_get() {
    local path="$1"
    if [[ "$RUSTERON_IMDS_ROOT" == http* ]]; then
        local token
        token="$(curl -fsS -X PUT "$RUSTERON_IMDS_ROOT/../api/token" \
            -H "X-aws-ec2-metadata-token-ttl-seconds: 21600" 2>/dev/null || true)"
        if [[ -n "$token" ]]; then
            curl -fsS -H "X-aws-ec2-metadata-token: $token" \
                "$RUSTERON_IMDS_ROOT/$path" 2>/dev/null || true
        else
            curl -fsS "$RUSTERON_IMDS_ROOT/$path" 2>/dev/null || true
        fi
    else
        # Local fixture path: a directory lists its children (like the IMDS
        # API's `macs/` listing); a file cats its contents.
        if [[ -d "$RUSTERON_IMDS_ROOT/$path" ]]; then
            ls -1 "$RUSTERON_IMDS_ROOT/$path" 2>/dev/null | tr '\n' ' '
        else
            cat "$RUSTERON_IMDS_ROOT/$path" 2>/dev/null || true
        fi
    fi
}

# --- 1. IOMMU ---------------------------------------------------------------
iommu_groups="$RUSTERON_SYSFS_ROOT/kernel/iommu_groups"
if [[ ! -d "$iommu_groups" ]] || [[ -z "$(ls -A "$iommu_groups" 2>/dev/null)" ]]; then
    fail "IOMMU groups missing/empty at $iommu_groups — enable IOMMU (iommu=pt) on this Nitro node"
fi
ok "IOMMU groups present ($(ls "$iommu_groups" | wc -l | tr -d ' ') group(s))"

# --- 2. VFIO ----------------------------------------------------------------
vfio_pci="$RUSTERON_SYSFS_ROOT/bus/pci/drivers/vfio-pci"
[[ -d "$vfio_pci" ]] || fail "vfio-pci driver not bound/loaded (check modprobe vfio-pci)"
ok "vfio-pci driver available"

# vfio_iommu_type1 must be present — without it, vfio falls back to unsafe
# no-IOMMU mode, which must never back a kernel-bypass transport.
if [[ ! -d "$RUSTERON_SYSFS_ROOT/module/vfio_iommu_type1" ]]; then
    fail "vfio_iommu_type1 not loaded — vfio would run in unsafe no-IOMMU mode (modprobe vfio_iommu_type1)"
fi
ok "vfio_iommu_type1 loaded (real IOMMU backing)"

# --- 3. Hugepages -----------------------------------------------------------
huge_nr="$RUSTERON_SYSFS_ROOT/kernel/mm/hugepages/hugepages-2048kB/nr_hugepages"
nr="$(cat "$huge_nr" 2>/dev/null || echo 0)"
((nr > 0)) || fail "no 2 MiB hugepages reserved (sysctl vm.nr_hugepages > 0)"
if ! ((RUSTERON_SKIP_MOUNT_CHECK)) && ! mountpoint -q "$RUSTERON_HUGEPAGES_DIR" 2>/dev/null; then
    fail "$RUSTERON_HUGEPAGES_DIR is not a hugetlbfs mount (mount -t hugetlbfs -o pagesize=2M)"
fi
ok "2 MiB hugepages reserved ($nr) and mounted at $RUSTERON_HUGEPAGES_DIR"

# --- 4. NUMA ----------------------------------------------------------------
[[ -d "$RUSTERON_SYSFS_ROOT/devices/system/node/node0" ]] \
    || fail "NUMA topology not present — DPDK needs node-local hugepages/mempools"
ok "NUMA topology present"

# --- 5. ENA discovery and primary-ENA exclusion -----------------------------
# List ENA PCI devices (Amazon vendor 0x1d0f) that own a netdev.
ena_netdevs=()
while read -r iface; do [[ -n "$iface" ]] && ena_netdevs+=("$iface"); done < <(
    for dev in "$RUSTERON_SYSFS_ROOT"/bus/pci/devices/*/; do
        [[ -d "$dev" ]] || continue
        vendor="$(cat "${dev}vendor" 2>/dev/null || true)"
        [[ "$vendor" == "0x1d0f" ]] || continue
        for net in "$dev"/net/*; do
            [[ -d "$net" ]] && basename "$net"
        done
    done
)
(( ${#ena_netdevs[@]} >= 3 )) \
    || fail "expected >= 3 ENA netdevs (primary + sender + receiver), found ${#ena_netdevs[@]}"

# Identify the primary ENA by its default route (kernel-owned; carries EKS,
# DNS, image pulls, telemetry, and the kernel-UDP rollback path). Overridable
# for fixture tests.
primary_mac="$RUSTERON_PRIMARY_MAC"
if [[ -z "$primary_mac" ]]; then
    primary_mac="$(ip -o addr show primary 2>/dev/null | awk '{print $17}')"
fi
if [[ -z "$primary_mac" ]]; then
    # Fallback: the netdev on the default-route interface.
    ifname="$(ip route show default 2>/dev/null | awk '{print $5; exit}')"
    [[ -n "$ifname" ]] && primary_mac="$(cat "/sys/class/net/$ifname/address" 2>/dev/null || true)"
fi
[[ -n "$primary_mac" ]] || fail "cannot identify the primary (default-route) ENA"

# IMDS MAC list (device-number 0 is the primary; secondaries carry the DPDK
# pair). Only secondaries are DPDK candidates — never the primary.
imds_macs="$(imds_get network/interfaces/macs/)"
[[ -n "$imds_macs" ]] || fail "IMDS returned no network interfaces"
macs=()
while read -r mac; do [[ -n "$mac" ]] && macs+=("$mac"); done \
    <<<"$(echo "$imds_macs" | tr ' ' '\n' | sed '/^$/d')"

secondaries=()
for mac in "${macs[@]}"; do
    [[ "$mac" == "$primary_mac" ]] && continue          # primary ENA: excluded
    devnum="$(imds_get "network/interfaces/macs/$mac/device-number")"
    [[ "$devnum" == "0" ]] && continue                   # IMDS primary by device-number
    secondaries+=("$mac")
done
(( ${#secondaries[@]} >= 2 )) \
    || fail "need >= 2 secondary ENAs for the sender/receiver pair, found ${#secondaries[@]}"
ok "primary ENA ($primary_mac) preserved; ${#secondaries[@]} secondary ENA(s) available"

# --- 6. IP/gateway identity ------------------------------------------------
for mac in "${secondaries[@]}"; do
    ip="$(imds_get "network/interfaces/macs/$mac/local-ipv4s" | head -1)"
    cidr="$(imds_get "network/interfaces/macs/$mac/subnet-ipv4-cidr-block")"
    eni="$(imds_get "network/interfaces/macs/$mac/interface-id")"
    [[ -n "$ip" && -n "$cidr" && -n "$eni" ]] \
        || fail "secondary $mac missing IMDS identity (eni/ip/cidr)"
    ok "secondary $mac -> eni=$eni ip=$ip cidr=$cidr"
done

echo "preflight PASS"
