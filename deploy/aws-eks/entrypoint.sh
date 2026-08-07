#!/usr/bin/env bash
# Container entrypoint for the rusteron-media-driver DPDK ENA image (plan
# §10.4). Reads the pod's effective cpuset (kubelet CPU Manager `static`) and
# assigns distinct exclusive CPUs to the Aeron conductor, sender and receiver
# agents. Fails before starting Aeron if fewer than the required exclusive
# CPUs are present. Then execs the passed command (default: media_driver).
#
# The RUSTERON_DPDK_* PCI/CIDR/gateway values are injected by the device
# plugin at allocation time; the media_driver binary itself fails fast (and
# nonzero, never falling back to the socket driver) if any are missing — the
# entrypoint deliberately does not duplicate that validation.
set -euo pipefail

REQUIRED_CPUS="${REQUIRED_CPUS:-3}"   # conductor + sender + receiver

# --- effective cpuset -------------------------------------------------------
# cgroup v2 exposes it at /sys/fs/cgroup/cpuset.cpus.effective; fall back to
# /proc/self/status when running outside cgroup v2.
read_cpuset() {
    local cg
    if cg="$(cat /sys/fs/cgroup/cpuset.cpus.effective 2>/dev/null)"; then
        echo "$cg"
        return 0
    fi
    sed -n 's/^Cpus_allowed_list:[[:space:]]*//p' /proc/self/status
}

# Expand "0-3,7" into one CPU id per line, ascending.
expand_cpuset() {
    local range cpu lo hi
    IFS=',' read -r -a ranges <<<"$1"
    for range in "${ranges[@]}"; do
        if [[ "$range" == *-* ]]; then
            lo="${range%-*}"; hi="${range#*-}"
            for ((cpu = lo; cpu <= hi; cpu++)); do echo "$cpu"; done
        else
            echo "$range"
        fi
    done | sort -n
}

CPUSET="$(read_cpuset)"
if [[ -z "$CPUSET" ]]; then
    echo "rusteron entrypoint: cannot read effective cpuset" >&2
    exit 1
fi
mapfile -t CPUS < <(expand_cpuset "$CPUSET")

if ((${#CPUS[@]} < REQUIRED_CPUS)); then
    echo "rusteron entrypoint: effective cpuset has ${#CPUS[@]} CPU(s), need >= $REQUIRED_CPUS (conductor + sender + receiver)" >&2
    exit 1
fi

CONDUCTOR_CPU="${CPUS[0]}"
SENDER_CPU="${CPUS[1]}"
RECEIVER_CPU="${CPUS[2]}"

# --- Aeron dedicated-thread driver settings (plan §6.4) ---------------------
# Set only when the caller hasn't already provided them.
export AERON_THREADING_MODE="${AERON_THREADING_MODE:-DEDICATED}"
export AERON_CONDUCTOR_IDLE_STRATEGY="${AERON_CONDUCTOR_IDLE_STRATEGY:-spin}"
export AERON_SENDER_IDLE_STRATEGY="${AERON_SENDER_IDLE_STRATEGY:-spin}"
export AERON_RECEIVER_IDLE_STRATEGY="${AERON_RECEIVER_IDLE_STRATEGY:-spin}"
export AERON_CONDUCTOR_CPU_AFFINITY="${AERON_CONDUCTOR_CPU_AFFINITY:-$CONDUCTOR_CPU}"
export AERON_SENDER_CPU_AFFINITY="${AERON_SENDER_CPU_AFFINITY:-$SENDER_CPU}"
export AERON_RECEIVER_CPU_AFFINITY="${AERON_RECEIVER_CPU_AFFINITY:-$RECEIVER_CPU}"
export AERON_SENDER_WILDCARD_PORT_RANGE="${AERON_SENDER_WILDCARD_PORT_RANGE:-20000-20999}"
export AERON_RECEIVER_WILDCARD_PORT_RANGE="${AERON_RECEIVER_WILDCARD_PORT_RANGE:-21000-21999}"
export AERON_MTU_LENGTH="${AERON_MTU_LENGTH:-1408}"

# --- DPDK selector and file prefix ------------------------------------------
export RUSTERON_MEDIA_DRIVER_TRANSPORT="${RUSTERON_MEDIA_DRIVER_TRANSPORT:-dpdk-ena}"
if [[ -z "${RUSTERON_DPDK_FILE_PREFIX:-}" ]]; then
    export RUSTERON_DPDK_FILE_PREFIX="rusteron-dpdk-${HOSTNAME}"
fi

echo "rusteron entrypoint: transport=${RUSTERON_MEDIA_DRIVER_TRANSPORT} cpuset=${CPUSET} conductor=${CONDUCTOR_CPU} sender=${SENDER_CPU} receiver=${RECEIVER_CPU}"
exec "$@"
