#!/usr/bin/env bash
# Reboot-safe node bootstrap for the rusteron-media-driver DPDK ENA pair
# (plan §9, §10.2). Run by systemd at boot (rusteron-dpdk-ena.service) and
# idempotent on repeat runs:
#
#   1. Preflight the node (IOMMU/VFIO/hugepages/NUMA/ENA identity).
#   2. Discover the two secondary ENAs from IMDS; NEVER touch the primary
#      (default-route) ENA — it stays kernel-owned for EKS/DNS/telemetry and
#      is the kernel-UDP rollback path.
#   3. Bind each secondary ENA to vfio-pci (unbind ena -> driver_override ->
#      probe).
#   4. Write /var/lib/rusteron-dpdk/ena-pairs.json atomically, recording the
#      sender/receiver PCI, IOMMU group, ENI id, MAC, IPv4 CIDR, gateway,
#      NUMA node, and health state.
#   5. Tune the primary ENA for lowest-latency kernel traffic (ethtool
#      coalescing off + busy-poll sysctls) and warn when the kernel cmdline
#      lacks the isolation flags (isolcpus/nohz_full/max_cstate).
#
# Testable against fake fixtures: set RUSTERON_SYSFS_ROOT, RUSTERON_IMDS_ROOT
# and RUSTERON_STATE_DIR to fixture paths, and RUSTERON_DRY_RUN=1 to skip the
# real vfio bind. See node/test/test-bootstrap.sh.
#
# Accepts no arguments.
set -euo pipefail

RUSTERON_SYSFS_ROOT="${RUSTERON_SYSFS_ROOT:-/sys}"
RUSTERON_IMDS_ROOT="${RUSTERON_IMDS_ROOT:-http://169.254.169.254/latest/meta-data}"
RUSTERON_STATE_DIR="${RUSTERON_STATE_DIR:-/var/lib/rusteron-dpdk}"
RUSTERON_DRY_RUN="${RUSTERON_DRY_RUN:-0}"
INVENTORY="$RUSTERON_STATE_DIR/ena-pairs.json"

fail() { echo "bootstrap FAIL: $*" >&2; exit 1; }
ok()   { echo "bootstrap ok: $*"; }
warn() { echo "bootstrap warn: $*" >&2; }

# IMDSv2-aware fetch; a non-http root is a local fixture path.
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

# Extract the subnet prefix length from an ip/prefix CIDR.
cidr_prefix() { echo "${1##*/}"; }

ip_to_int() { local IFS=. a b c d; read -r a b c d <<<"$1"; echo $(( a*16777216 + b*65536 + c*256 + d )); }
int_to_ip() { printf "%d.%d.%d.%d\n" $(( ($1>>24)&255 )) $(( ($1>>16)&255 )) $(( ($1>>8)&255 )) $(( $1&255 )); }

# Compute the VPC subnet gateway (network address + 1) from an ip/prefix CIDR.
# Pure bash arithmetic (macOS BSD awk has no bitwise ops). Network = top of the
# /N page, gateway = network + 1 (AWS VPC convention).
cidr_gateway() {
    local cidr="$1" prefix host_bits page
    prefix="${cidr##*/}"
    host_bits=$((32-prefix))
    page=$((1<<host_bits))
    int_to_ip $(( (( $(ip_to_int "${cidr%/*}") / page )) * page + 1 ))
}

# Resolve the PCI BDF owning a given MAC by scanning sysfs net devices.
mac_to_bdf() {
    local mac="$1" dev net
    for dev in "$RUSTERON_SYSFS_ROOT"/bus/pci/devices/*/; do
        [[ -d "$dev" ]] || continue
        for net in "$dev"/net/*; do
            [[ -d "$net" ]] || continue
            [[ "$(cat "$net/address" 2>/dev/null || true)" == "$mac" ]] \
                && echo "$(basename "$dev")" && return 0
        done
    done
    return 1
}

# Bind one ENA BDF to vfio-pci. Skipped under dry-run (fixture tests).
bind_vfio() {
    local bdf="$1"
    local pci="$RUSTERON_SYSFS_ROOT/bus/pci/devices/$bdf"
    local ena_drv="$RUSTERON_SYSFS_ROOT/bus/pci/drivers/ena"
    [[ -d "$pci" ]] || fail "no PCI device $bdf in sysfs"
    [[ -e "$pci/driver" ]] && [[ "$(readlink "$pci/driver")" != */vfio-pci ]] || return 0  # already bound

    if ((RUSTERON_DRY_RUN)); then
        ok "dry-run: would bind $bdf to vfio-pci"
        return 0
    fi

    if [[ -d "$ena_drv" ]] && [[ -e "$pci/driver" ]]; then
        echo "$bdf" > "$ena_drv/unbind"
    fi
    echo "vfio-pci" > "$pci/driver_override"
    echo "$bdf" > "$RUSTERON_SYSFS_ROOT/bus/pci/drivers_probe"
    ok "bound $bdf to vfio-pci"
}

# --- 1. preflight (exit 1 before any change if the node is unsafe) ----------
# shellcheck source=preflight.sh
source "$(dirname "$(realpath "${BASH_SOURCE[0]}")")/preflight.sh"

# --- 2. discover secondaries from IMDS --------------------------------------
# Primary ENA: honored override (fixture tests) or default-route interface.
# The preflight sourced above already ran the same detection; recompute here so
# the bootstrap is self-contained.
primary_mac="$RUSTERON_PRIMARY_MAC"
if [[ -z "$primary_mac" ]]; then
    primary_mac="$(ip -o addr show primary 2>/dev/null | awk '{print $17}')"
fi
if [[ -z "$primary_mac" ]]; then
    ifname="$(ip route show default 2>/dev/null | awk '{print $5; exit}')"
    [[ -n "$ifname" ]] && primary_mac="$(cat "/sys/class/net/$ifname/address" 2>/dev/null || true)"
fi
[[ -n "$primary_mac" ]] || fail "cannot identify the primary (default-route) ENA"

imds_macs="$(imds_get network/interfaces/macs/)"
[[ -n "$imds_macs" ]] || fail "IMDS returned no network interfaces"
macs=()
while read -r mac; do [[ -n "$mac" ]] && macs+=("$mac"); done \
    <<<"$(echo "$imds_macs" | tr ' ' '\n' | sed '/^$/d')"

secondaries=()
for mac in "${macs[@]}"; do
    [[ "$mac" == "$primary_mac" ]] && continue
    devnum="$(imds_get "network/interfaces/macs/$mac/device-number")"
    [[ "$devnum" == "0" ]] && continue
    secondaries+=("$mac")
done
(( ${#secondaries[@]} >= 2 )) \
    || fail "need >= 2 secondary ENAs (sender + receiver), found ${#secondaries[@]}"
# Deterministic role order (sorted), independent of IMDS ordering.
secondaries=( $(printf '%s\n' "${secondaries[@]}" | sort) )
SENDER_MAC="${secondaries[0]}"
RECEIVER_MAC="${secondaries[1]}"
ok "primary ENA $primary_mac preserved; sender=$SENDER_MAC receiver=$RECEIVER_MAC"

# --- 3. bind both secondaries to vfio-pci -----------------------------------
pair_record() {
    local role="$1" mac="$2"
    local bdf group eni ip cidr gw numa
    bdf="$(mac_to_bdf "$mac")" || fail "no PCI BDF found for $role ENA $mac"
    group="$(basename "$(readlink "$RUSTERON_SYSFS_ROOT/bus/pci/devices/$bdf/iommu_group" 2>/dev/null || true)" 2>/dev/null || true)"
    [[ -n "$group" ]] || fail "$role ENA $bdf has no IOMMU group"
    eni="$(imds_get "network/interfaces/macs/$mac/interface-id")"
    ip="$(imds_get "network/interfaces/macs/$mac/local-ipv4s" | head -1)"
    subnet="$(imds_get "network/interfaces/macs/$mac/subnet-ipv4-cidr-block")"
    prefix="$(cidr_prefix "$subnet")"
    gw="$(cidr_gateway "$subnet")"
    numa="$(cat "$RUSTERON_SYSFS_ROOT/bus/pci/devices/$bdf/numa_node" 2>/dev/null || echo -1)"
    [[ -n "$eni" && -n "$ip" && -n "$subnet" && -n "$prefix" ]] \
        || fail "$role ENA $mac missing IMDS identity (eni/ip/subnet)"
    bind_vfio "$bdf"
    printf '  "%s": {"pci":"%s","iommu_group":"%s","eni_id":"%s","mac":"%s",' \
        "$role" "$bdf" "$group" "$eni" "$mac"
    printf '"ipv4":"%s","prefix_len":%s,"subnet_cidr":"%s","gateway":"%s","numa_node":%s,"health":"healthy"}' \
        "$ip" "$prefix" "$subnet" "$gw" "$numa"
}

# --- 4. write inventory atomically ------------------------------------------
mkdir -p "$RUSTERON_STATE_DIR"
tmp="$INVENTORY.tmp.$$"
{
    echo '{'
    echo '  "generated_at": "'"$(date -u +%Y-%m-%dT%H:%M:%SZ)"'",'
    echo '  "primary_ena_mac": "'"$primary_mac"'",'
    echo '  "pairs": ['
    echo '    {'
    echo '      "id": "dpdk-pair-0",'
    pair_record sender   "$SENDER_MAC"
    echo ','
    pair_record receiver "$RECEIVER_MAC"
    echo ''
    echo '    }'
    echo '  ]'
    echo '}'
} > "$tmp"
mv "$tmp" "$INVENTORY"
chmod 0644 "$INVENTORY"
ok "wrote $INVENTORY atomically"

# --- 5. node latency tuning (best-effort; never fatal) ----------------------
# Lowest-latency kernel path on the primary ENA (plan §12 latency levers:
# IRQ coalescing off + busy-poll on the kernel/rollback path), plus a soft
# check that the kernel cmdline carries the isolation flags (isolcpus/nohz_full
# can only be applied at node-provision time — needs a replace, so warn).
tune_primary_ena() {
    local ifname="$1"
    if ! command -v ethtool >/dev/null 2>&1; then
        warn "ethtool not installed — skipping primary-ENA IRQ coalescing tuning"
        return 0
    fi
    [[ -n "$ifname" ]] || { warn "cannot resolve primary ENA netdev — skipping ethtool tuning"; return 0; }
    if ((RUSTERON_DRY_RUN)); then
        ok "dry-run: would tune $ifname (adaptive-rx off, rx/tx-usecs 0, busy_poll/read=70)"
        return 0
    fi
    ethtool -C "$ifname" adaptive-rx off 2>/dev/null || true
    ethtool -C "$ifname" rx-usecs 0 tx-usecs 0 2>/dev/null || true
    ethtool -C "$ifname" rx-frames 0 tx-frames 0 2>/dev/null || true
    sysctl -q -w net.core.busy_poll=70 net.core.busy_read=70 2>/dev/null || true
    ok "tuned $ifname: adaptive-rx off, rx/tx coalescing disabled, busy_poll/read=70"
}

check_kernel_cmdline() {
    local cmdline missing=()
    cmdline="$(cat /proc/cmdline 2>/dev/null || true)"
    for w in isolcpus nohz_full max_cstate; do
        [[ "$cmdline" == *"$w="* ]] || missing+=("$w")
    done
    ((${#missing[@]})) && warn "kernel cmdline lacks ${missing[*]} — add for the DPDK pod's CPU set (needs node replace; runbook §2)"
}

primary_ifname=""
for d in "$RUSTERON_SYSFS_ROOT"/class/net/*; do
    [[ -d "$d" ]] || continue
    if [[ "$(cat "$d/address" 2>/dev/null || true)" == "$primary_mac" ]]; then
        primary_ifname="$(basename "$d")"
        break
    fi
done
tune_primary_ena "$primary_ifname"
check_kernel_cmdline
