#!/usr/bin/env bash
# Fixture test for node/bootstrap-dpdk-ena.sh + node/preflight.sh (plan §9:
# "Test scripts against fake sysfs/IMDS fixtures before live nodes").
#
# Builds a fake sysfs tree (three Amazon ENA PCI devices: primary + sender +
# receiver) and a fake IMDS tree, then runs preflight (must PASS) and
# bootstrap (must write a complete ena-pairs.json under RUSTERON_DRY_RUN so no
# host sysfs is touched). Also verifies the unsafe-I-OMMU preflight failure.
#
# Run from anywhere:  bash node/test/test-bootstrap.sh
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NODE_DIR="$HERE/.."
PREFLIGHT="$NODE_DIR/preflight.sh"
BOOTSTRAP="$NODE_DIR/bootstrap-dpdk-ena.sh"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
SYSFS="$TMP/sysfs"
IMDS="$TMP/imds"
STATE="$TMP/state"

# Primary is on eth0/device 0; sender eth1; receiver eth2. All are Amazon ENAs.
PRIMARY_MAC="02:00:00:00:00:01"
SENDER_MAC="02:00:00:00:00:02"
RECEIVER_MAC="02:00:00:00:00:03"

make_pci() { # bdf iface mac numa iommu_group
    local bdf="$1" iface="$2" mac="$3" numa="$4" grp="$5"
    local d="$SYSFS/bus/pci/devices/$bdf"
    mkdir -p "$d/net/$iface"
    echo "0x1d0f" > "$d/vendor"                       # Amazon
    echo "$mac"  > "$d/net/$iface/address"
    echo "$numa" > "$d/numa_node"
    mkdir -p "$SYSFS/kernel/iommu_groups/$grp"
    ln -s "../../../../kernel/iommu_groups/$grp" "$d/iommu_group"
}

make_eni() { # mac device-number eni ip cidr
    local mac="$1" devnum="$2" eni="$3" ip="$4" cidr="$5"
    local d="$IMDS/network/interfaces/macs/$mac"
    mkdir -p "$d"
    echo "$devnum" > "$d/device-number"
    echo "$eni"    > "$d/interface-id"
    echo "$ip"     > "$d/local-ipv4s"
    echo "$cidr"   > "$d/subnet-ipv4-cidr-block"
}

# --- fake sysfs -------------------------------------------------------------
mkdir -p "$SYSFS/kernel/iommu_groups" "$SYSFS/bus/pci/drivers/vfio-pci"
mkdir -p "$SYSFS/module/vfio_iommu_type1"
mkdir -p "$SYSFS/kernel/mm/hugepages/hugepages-2048kB"
mkdir -p "$SYSFS/devices/system/node/node0"
echo 512 > "$SYSFS/kernel/mm/hugepages/hugepages-2048kB/nr_hugepages"
make_pci 0000:00:05.0 eth0 "$PRIMARY_MAC" 0 10   # primary (excluded)
make_pci 0000:00:06.0 eth1 "$SENDER_MAC"   0 11   # sender
make_pci 0000:00:07.0 eth2 "$RECEIVER_MAC" 0 12   # receiver

# --- fake IMDS --------------------------------------------------------------
make_eni "$PRIMARY_MAC" 0 eni-1234567890abcdef0 10.0.0.10   10.0.0.0/24
make_eni "$SENDER_MAC"   1 eni-1234567890abcdef1 10.0.1.10   10.0.1.0/24
make_eni "$RECEIVER_MAC" 2 eni-1234567890abcdef2 10.0.2.10   10.0.2.0/24

# --- 1. preflight PASS ------------------------------------------------------
echo "== preflight (healthy fixture) =="
RUSTERON_SYSFS_ROOT="$SYSFS" \
RUSTERON_IMDS_ROOT="$IMDS" \
RUSTERON_HUGEPAGES_DIR="$TMP/hugepages" \
RUSTERON_SKIP_MOUNT_CHECK=1 \
RUSTERON_PRIMARY_MAC="$PRIMARY_MAC" \
    "$PREFLIGHT"
echo "preflight PASSED (as expected)"

# --- 2. bootstrap writes a complete inventory -------------------------------
echo "== bootstrap (dry run) =="
RUSTERON_SYSFS_ROOT="$SYSFS" \
RUSTERON_IMDS_ROOT="$IMDS" \
RUSTERON_STATE_DIR="$STATE" \
RUSTERON_HUGEPAGES_DIR="$TMP/hugepages" \
RUSTERON_SKIP_MOUNT_CHECK=1 \
RUSTERON_PRIMARY_MAC="$PRIMARY_MAC" \
RUSTERON_DRY_RUN=1 \
    "$BOOTSTRAP"

INVENTORY="$STATE/ena-pairs.json"
[[ -f "$INVENTORY" ]] || { echo "FAIL: inventory not written" >&2; exit 1; }
echo "--- inventory ---"
cat "$INVENTORY"

# Assertions: primary preserved, both roles present with full identity.
for needle in \
    "primary_ena_mac.*$PRIMARY_MAC" \
    '"pci":"0000:00:06.0"' \
    '"pci":"0000:00:07.0"' \
    "eni-1234567890abcdef1" "eni-1234567890abcdef2" \
    '"ipv4":"10.0.1.10"' \
    '"ipv4":"10.0.2.10"' \
    '"prefix_len":24' \
    '"subnet_cidr":"10.0.1.0/24"' \
    '"subnet_cidr":"10.0.2.0/24"' \
    '"gateway":"10.0.1.1"' \
    '"gateway":"10.0.2.1"' \
    '"iommu_group":"11"' \
    '"iommu_group":"12"' \
    '"numa_node":0' \
    '"health":"healthy"' \
    ; do
    grep -q "$needle" "$INVENTORY" || { echo "FAIL: inventory missing '$needle'" >&2; exit 1; }
done
echo "inventory assertions PASSED"

# --- 3. unsafe IOMMU fails preflight ----------------------------------------
echo "== preflight (unsafe: no vfio_iommu_type1) =="
rm -rf "$SYSFS/module/vfio_iommu_type1"
if RUSTERON_SYSFS_ROOT="$SYSFS" \
    RUSTERON_IMDS_ROOT="$IMDS" \
    RUSTERON_HUGEPAGES_DIR="$TMP/hugepages" \
    RUSTERON_SKIP_MOUNT_CHECK=1 \
    RUSTERON_PRIMARY_MAC="$PRIMARY_MAC" \
        "$PREFLIGHT" 2>/dev/null; then
    echo "FAIL: preflight should have failed without vfio_iommu_type1" >&2
    exit 1
fi
echo "unsafe-IOMMU preflight FAILED (as expected)"

echo "ALL TESTS PASSED"
