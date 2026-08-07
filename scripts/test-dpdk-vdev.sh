#!/usr/bin/env bash
# rusteron-media-driver DPDK vdev interoperability harness (plan §11.2).
#
# Runs the dpdk-harness scenarios over DPDK tap devices on a Linux bridge,
# exercising DPDK-to-DPDK, DPDK-to-default-UDP in both directions, reconnect,
# multiple endpoints, loss recovery, and restart cleanup.
#
# This requires a PRIVILEGED Linux x86_64 host:
#   - root (the script creates bridges/taps and mounts hugetlbfs)
#   - /dev/net/tun available (modprobe tun)
#   - libdpdk dev packages (matching the DPDK version the build links)
#   - a Rust toolchain (cargo)
#
# The DPDK tap PMD assigns the host interface names dynamically (dtapN), so
# this script discovers the interfaces each process creates rather than
# assuming a fixed name.
#
# Usage:
#   sudo scripts/test-dpdk-vdev.sh [--scenario NAME] [--msgs N]

set -euo pipefail
cd "$(dirname "$0")/.."

BRIDGE=rusteron-vdev0
BRIDGE_IP=10.9.0.5          # the default-UDP side binds here
DPDK_SUBNET=10.9.0.0/24
GW=10.9.0.254
HUGEDIR=/dev/hugepages
REPORT_DIR="${RUSTERON_REPORT_DIR:-/tmp/rusteron-vdev-reports}"
WORK_DIR="${RUSTERON_WORK_DIR:-/tmp/rusteron-vdev}"
CARGO="${CARGO:-cargo}"

SCENARIO=""
MSGS=1000
CLEANUP_ON_EXIT=1

log()  { echo "[vdev] $*" >&2; }
die()  { echo "[vdev] ERROR: $*" >&2; cleanup; exit 1; }

usage() {
    sed -n '2,20p' "$0"
    echo "  --scenario NAME   run only NAME (default: all)"
    echo "  --msgs N          messages per run (default: 1000)"
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --scenario) SCENARIO="$2"; shift 2 ;;
        --msgs)     MSGS="$2"; shift 2 ;;
        -h|--help)  usage ;;
        *) die "unknown argument $1" ;;
    esac
done

ALL_SCENARIOS="bidirectional_unicast dpdk_to_udp udp_to_dpdk reconnect multi_endpoint loss_recovery restart"

# ---------------------------------------------------------------------------
# Prerequisites
# ---------------------------------------------------------------------------

[[ "$(id -u)" -eq 0 ]] || die "must run as root (sudo scripts/test-dpdk-vdev.sh)"
[[ -e /dev/net/tun ]] || die "/dev/net/tun missing — modprobe tun"
command -v "$CARGO" >/dev/null || die "cargo not found"
command -v pkg-config >/dev/null || die "pkg-config not found"
pkg-config --exists libdpdk || die "libdpdk dev package not found (pkg-config libdpdk)"
pkg-config --exists libdpdk >/dev/null || true

setup_hugepages() {
    if ! mountpoint -q "$HUGEDIR"; then
        mkdir -p "$HUGEDIR"
        mount -t hugetlbfs hugetlbfs "$HUGEDIR" || die "failed to mount hugetlbfs at $HUGEDIR"
    fi
    local pages; pages=$(sysctl -n vm.nr_hugepages 2>/dev/null || echo 0)
    if (( pages < 256 )); then
        log "raising vm.nr_hugepages 0 -> 256"
        sysctl -w vm.nr_hugepages=256 >/dev/null
    fi
}

build_harness() {
    if [[ -x "${RUSTERON_HARNESS_BIN:-}" ]]; then
        HARNESS_BIN="$RUSTERON_HARNESS_BIN"
        log "using prebuilt harness $HARNESS_BIN"
        return
    fi
    log "building dpdk-harness (--features dpdk)"
    "$CARGO" build -p dpdk-harness --features dpdk
    HARNESS_BIN="target/debug/dpdk-harness"
    [[ -x "$HARNESS_BIN" ]] || die "harness build did not produce $HARNESS_BIN"
}

# ---------------------------------------------------------------------------
# Bridge + tap discovery
# ---------------------------------------------------------------------------

snapshot_taps() {
    ip -o link show 2>/dev/null | awk -F': ' '{print $2}' | grep -E '^(dtap|tap)[0-9]+$' | sort -u
}

# new_taps <snapshot-file>: interfaces in snapshot-file's "before" list that
# have appeared since (write current list into $TAPS for the caller).
discover_taps() {
    local before=$1 pid=$2 want=$3 timeout=${4:-25} after
    local deadline=$((SECONDS + timeout))
    while (( SECONDS < deadline )); do
        if ! kill -0 "$pid" 2>/dev/null; then
            return 1   # process died before creating its taps
        fi
        after=$(snapshot_taps)
        TAPS=$(comm -13 <(sort "$before") <(echo "$after"))
        local n; n=$(echo "$TAPS" | grep -c . || true)
        if (( n >= want )); then
            echo "$TAPS" | head -n "$want"
            return 0
        fi
        sleep 0.5
    done
    return 1
}

bridge_up() {
    if ! ip link show "$BRIDGE" >/dev/null 2>&1; then
        ip link add "$BRIDGE" type bridge
        ip addr add "$BRIDGE_IP/24" dev "$BRIDGE"
        ip link set "$BRIDGE" up
        # Allow the bridge itself to answer ARP for $BRIDGE_IP and L3-terminate.
        echo 0 > /proc/sys/net/bridge/bridge-nf-call-iptables 2>/dev/null || true
    fi
    log "bridge $BRIDGE up at $BRIDGE_IP/24"
}

# attach_tap <iface> [ip]: add to the bridge and (optionally) assign an IP.
attach_tap() {
    local iface=$1 ip=$2
    ip link set "$iface" up
    ip link set "$iface" master "$BRIDGE"
    if [[ -n "$ip" ]]; then
        ip addr add "$ip/24" dev "$iface"
    fi
    log "tap $iface -> bridge${ip:+ @ $ip}"
}

# ---------------------------------------------------------------------------
# Environment for a harness process
# ---------------------------------------------------------------------------

dpdk_env() {
    local role=$1 suffix
    suffix=rusteron-vdev-p
    [[ "$role" == secondary ]] && suffix=rusteron-vdev-s
    echo "RUSTERON_MEDIA_DRIVER_TRANSPORT=dpdk-ena"
    echo "RUSTERON_DPDK_FILE_PREFIX=$suffix"
    echo "RUSTERON_DPDK_SENDER_PCI=net_tap0"
    echo "RUSTERON_DPDK_RECEIVER_PCI=net_tap1"
    echo "RUSTERON_DPDK_HUGE_DIR=$HUGEDIR"
    echo "RUSTERON_DPDK_RX_DESCRIPTORS=1024"
    echo "RUSTERON_DPDK_TX_DESCRIPTORS=1024"
    echo "RUSTERON_DPDK_MBUFS_PER_PORT=65536"
    echo "RUSTERON_DPDK_MEMPOOL_CACHE=256"
    echo "RUSTERON_DPDK_BURST_SIZE=32"
    echo "RUSTERON_DPDK_MAX_AERON_MTU=1408"
}

harness_common_env() {
    echo "RUSTERON_HARNESS_MSGS=$MSGS"
    echo "RUSTERON_HARNESS_PAYLOAD=130"
    echo "RUSTERON_HARNESS_STREAM=32931"
    echo "RUSTERON_HARNESS_TIMEOUT_SECS=40"
    echo "RUSTERON_HARNESS_MTU=1408"
    echo "RUSTERON_HARNESS_SENDER_CPU=${RUSTERON_HARNESS_SENDER_CPU:-1}"
    echo "RUSTERON_HARNESS_RECEIVER_CPU=${RUSTERON_HARNESS_RECEIVER_CPU:-2}"
}

# run_one <role> <scenario> <run-id> <envfile> [extra args...]
run_one() {
    local role=$1 scenario=$2 run_id=$3 envfile=$4
    shift 4
    local report="$REPORT_DIR/$scenario-$role-$run_id.json"
    log "launching $role ($scenario, run $run_id)"
    # shellcheck disable=SC2046
    env $(cat "$envfile") "$HARNESS_BIN" --role "$role" --scenario "$scenario" --report "$report" "$@" \
        >"$WORK_DIR/$scenario-$role-$run_id.log" 2>&1 &
    echo "$report $!"
}

wait_report() {
    local report=$1 pid=$2
    local deadline=$((SECONDS + 45))
    while (( SECONDS < deadline )); do
        if [[ -s "$report" ]]; then
            return 0
        fi
        if ! kill -0 "$pid" 2>/dev/null; then
            log "process for $report exited before writing a report"
            return 1
        fi
        sleep 0.5
    done
    return 1
}

report_field() {
    local report=$1 field=$2
    grep -o "\"$field\": [^,}]*" "$report" | head -1 | cut -d' ' -f2
}

assert_report() {
    local report=$1 expect_received=$2 label=$3
    local ok; ok=$(report_field "$report" ok)
    local sent; sent=$(report_field "$report" sent)
    local received; received=$(report_field "$report" received)
    log "assert $label: ok=$ok sent=$sent received=$received"
    [[ "$ok" == true ]] || die "$label: report not ok"
    [[ "$received" == "$expect_received" ]] || die "$label: received $received != $expect_received"
    echo "$sent"
}

# ---------------------------------------------------------------------------
# Scenario runners
# ---------------------------------------------------------------------------

# scenario_bidi <scenario> <primary-transport> <secondary-transport>
#   transport: dpdk | udp
scenario_bidi() {
    local scenario=$1 ptx=$2 stx=$3
    local penv=$WORK_DIR/$scenario-p.env senv=$WORK_DIR/$scenario-s.env
    : > "$penv"; : > "$senv"

    harness_common_env >> "$penv"; harness_common_env >> "$senv"

    # Primary: sender tap 10.9.0.1, receiver tap 10.9.0.3; sends to secondary
    # receiver 10.9.0.4:40102, receives on 10.9.0.3:40102.
    echo "RUSTERON_HARNESS_PUB_CTRL=10.9.0.1:40101" >> "$penv"
    echo "RUSTERON_HARNESS_SUB_ENDPOINTS=10.9.0.3:40102" >> "$penv"
    # Secondary: sender 10.9.0.2, receiver 10.9.0.4; sends to primary 10.9.0.3.
    echo "RUSTERON_HARNESS_PUB_CTRL=10.9.0.2:40101" >> "$senv"
    echo "RUSTERON_HARNESS_SUB_ENDPOINTS=10.9.0.4:40102" >> "$senv"

    if [[ "$ptx" == dpdk ]]; then
        dpdk_env primary >> "$penv"
        echo "RUSTERON_HARNESS_DESTINATIONS=10.9.0.4:40102" >> "$penv"
    else
        echo "RUSTERON_HARNESS_PUB_CTRL=$BRIDGE_IP:40101" >> "$penv"
        echo "RUSTERON_HARNESS_SUB_ENDPOINTS=$BRIDGE_IP:40103" >> "$penv"
        echo "RUSTERON_HARNESS_DESTINATIONS=10.9.0.3:40102" >> "$penv"
    fi
    if [[ "$stx" == dpdk ]]; then
        dpdk_env secondary >> "$senv"
        echo "RUSTERON_HARNESS_DESTINATIONS=10.9.0.3:40102" >> "$senv"
    else
        echo "RUSTERON_HARNESS_DESTINATIONS=10.9.0.3:40102" >> "$senv"
    fi

    # Launch primary, discover its taps (sender first, receiver second).
    local before_p; before_p=$(snapshot_taps)
    local p_spec; p_spec=$(run_one primary "$scenario" 1 "$penv")
    local p_report=${p_spec%% *} p_pid=${p_spec##* }
    local p_taps; p_taps=$(discover_taps "$before_p" "$p_pid" 2) || die "primary ($ptx) tap discovery failed"
    p_taps=( $p_taps )
    if [[ "$ptx" == dpdk ]]; then
        attach_tap "${p_taps[0]}" 10.9.0.1
        attach_tap "${p_taps[1]}" 10.9.0.3
    else
        : # UDP side uses the bridge interface itself
    fi

    # Launch secondary, discover its taps.
    local before_s; before_s=$(snapshot_taps)
    local s_spec; s_spec=$(run_one secondary "$scenario" 1 "$senv")
    local s_report=${s_spec%% *} s_pid=${s_spec##* }
    local s_taps; s_taps=$(discover_taps "$before_s" "$s_pid" 2) || die "secondary ($stx) tap discovery failed"
    s_taps=( $s_taps )
    if [[ "$stx" == dpdk ]]; then
        attach_tap "${s_taps[0]}" 10.9.0.2
        attach_tap "${s_taps[1]}" 10.9.0.4
    fi

    # Secondary's receiver tap gets the loss injection for the loss scenario.
    if [[ "$scenario" == loss_recovery ]]; then
        local rxtap
        if [[ "$stx" == dpdk ]]; then rxtap="${s_taps[1]}"; else rxtap=""; fi
        if [[ -n "$rxtap" ]]; then
            tc qdisc add dev "$rxtap" root netem loss 3% 25%
            log "netem 3% loss on $rxtap (receiver)"
        else
            tc qdisc add dev "$BRIDGE" root netem loss 3% 25%
            log "netem 3% loss on bridge (UDP receiver)"
        fi
    fi

    wait_report "$p_report" "$p_pid" || die "$scenario primary did not report"
    wait_report "$s_report" "$s_pid" || die "$scenario secondary did not report"

    assert_report "$p_report" "$MSGS" "primary->secondary"
    assert_report "$s_report" "$MSGS" "secondary->primary"

    kill "$p_pid" 2>/dev/null || true
    kill "$s_pid" 2>/dev/null || true
    wait "$p_pid" 2>/dev/null || true
    wait "$s_pid" 2>/dev/null || true
    log "scenario $scenario PASSED"
}

scenario_reconnect() {
    local scenario=reconnect
    local penv=$WORK_DIR/$scenario-p.env senv=$WORK_DIR/$scenario-s.env
    : > "$penv"; : > "$senv"
    harness_common_env >> "$penv"; harness_common_env >> "$senv"
    dpdk_env primary >> "$penv"; dpdk_env secondary >> "$senv"

    # Primary reconnects from 10.9.0.4:40102 to 10.9.0.4:40104.
    echo "RUSTERON_HARNESS_PUB_CTRL=10.9.0.1:40101" >> "$penv"
    echo "RUSTERON_HARNESS_PUB_CTRL2=10.9.0.1:40111" >> "$penv"
    echo "RUSTERON_HARNESS_DESTINATIONS=10.9.0.4:40102,10.9.0.4:40104" >> "$penv"
    # Secondary registers both endpoints.
    echo "RUSTERON_HARNESS_SUB_ENDPOINTS=10.9.0.4:40102,10.9.0.4:40104" >> "$senv"
    echo "RUSTERON_HARNESS_PUB_CTRL=10.9.0.2:40101" >> "$senv"
    echo "RUSTERON_HARNESS_DESTINATIONS=10.9.0.3:40102" >> "$senv"

    local before_p; before_p=$(snapshot_taps)
    local p_spec; p_spec=$(run_one primary "$scenario" 1 "$penv")
    local p_report=${p_spec%% *} p_pid=${p_spec##* }
    local p_taps; p_taps=$(discover_taps "$before_p" "$p_pid" 2) || die "primary tap discovery failed"
    p_taps=( $p_taps )
    attach_tap "${p_taps[0]}" 10.9.0.1
    attach_tap "${p_taps[1]}" 10.9.0.3

    local before_s; before_s=$(snapshot_taps)
    local s_spec; s_spec=$(run_one secondary "$scenario" 1 "$senv")
    local s_report=${s_spec%% *} s_pid=${s_spec##* }
    local s_taps; s_taps=$(discover_taps "$before_s" "$s_pid" 2) || die "secondary tap discovery failed"
    s_taps=( $s_taps )
    attach_tap "${s_taps[0]}" 10.9.0.2
    attach_tap "${s_taps[1]}" 10.9.0.4

    wait_report "$p_report" "$p_pid" || die "reconnect primary did not report"
    wait_report "$s_report" "$s_pid" || die "reconnect secondary did not report"

    assert_report "$p_report" 0 "reconnect primary (sender)"
    assert_report "$s_report" "$MSGS" "reconnect secondary (received both batches)"

    kill "$p_pid" 2>/dev/null || true
    kill "$s_pid" 2>/dev/null || true
    wait "$p_pid" 2>/dev/null || true
    wait "$s_pid" 2>/dev/null || true
    log "scenario $scenario PASSED"
}

scenario_multi_endpoint() {
    local scenario=multi_endpoint
    local penv=$WORK_DIR/$scenario-p.env senv=$WORK_DIR/$scenario-s.env
    : > "$penv"; : > "$senv"
    harness_common_env >> "$penv"; harness_common_env >> "$senv"
    dpdk_env primary >> "$penv"; dpdk_env secondary >> "$senv"

    # Primary publishes to all three secondary endpoints.
    echo "RUSTERON_HARNESS_PUB_CTRL=10.9.0.1:40101" >> "$penv"
    echo "RUSTERON_HARNESS_DESTINATIONS=10.9.0.4:40102,10.9.0.4:40112,10.9.0.4:40122" >> "$penv"
    echo "RUSTERON_HARNESS_SUB_ENDPOINTS=10.9.0.4:40102,10.9.0.4:40112,10.9.0.4:40122" >> "$senv"
    echo "RUSTERON_HARNESS_PUB_CTRL=10.9.0.2:40101" >> "$senv"
    echo "RUSTERON_HARNESS_DESTINATIONS=10.9.0.3:40102" >> "$senv"

    local before_p; before_p=$(snapshot_taps)
    local p_spec; p_spec=$(run_one primary "$scenario" 1 "$penv")
    local p_report=${p_spec%% *} p_pid=${p_spec##* }
    local p_taps; p_taps=$(discover_taps "$before_p" "$p_pid" 2) || die "primary tap discovery failed"
    p_taps=( $p_taps )
    attach_tap "${p_taps[0]}" 10.9.0.1
    attach_tap "${p_taps[1]}" 10.9.0.3

    local before_s; before_s=$(snapshot_taps)
    local s_spec; s_spec=$(run_one secondary "$scenario" 1 "$senv")
    local s_report=${s_spec%% *} s_pid=${s_spec##* }
    local s_taps; s_taps=$(discover_taps "$before_s" "$s_pid" 2) || die "secondary tap discovery failed"
    s_taps=( $s_taps )
    attach_tap "${s_taps[0]}" 10.9.0.2
    attach_tap "${s_taps[1]}" 10.9.0.4

    wait_report "$p_report" "$p_pid" || die "multi_endpoint primary did not report"
    wait_report "$s_report" "$s_pid" || die "multi_endpoint secondary did not report"

    # Secondary expects msgs * 3 endpoints.
    local expect=$((MSGS * 3))
    assert_report "$p_report" 0 "multi_endpoint primary (sender)"
    assert_report "$s_report" "$expect" "multi_endpoint secondary (3 endpoints)"

    kill "$p_pid" 2>/dev/null || true
    kill "$s_pid" 2>/dev/null || true
    wait "$p_pid" 2>/dev/null || true
    wait "$s_pid" 2>/dev/null || true
    log "scenario $scenario PASSED"
}

scenario_restart() {
    local scenario=restart
    log "scenario $scenario (run 1)"
    scenario_bidi bidirectional_unicast dpdk dpdk

    # Restart: kill the primary and verify it frees its taps and file-prefix
    # lock, then relaunch and run again.
    log "scenario $scenario: restarting primary"
    local pids; pids=$(pgrep -f "dpdk-harness --role primary" || true)
    if [[ -n "$pids" ]]; then
        kill $pids 2>/dev/null || true
        sleep 2
        pids=$(pgrep -f "dpdk-harness --role primary" || true)
        [[ -z "$pids" ]] || die "primary did not exit on SIGTERM"
    fi
    # Any stale tap devices from the killed primary must be gone.
    local stale; stale=$(snapshot_taps)
    [[ -z "$stale" ]] || log "note: $stale tap devices remain after kill"

    log "scenario $scenario (run 2)"
    scenario_bidi bidirectional_unicast dpdk dpdk
    log "scenario $scenario PASSED"
}

# ---------------------------------------------------------------------------
# Cleanup + main
# ---------------------------------------------------------------------------

cleanup() {
    [[ -n "${CLEANUP_ON_EXIT:-}" ]] || return 0
    pkill -f 'dpdk-harness' 2>/dev/null || true
    sleep 1
    ip link del "$BRIDGE" 2>/dev/null || true
    for t in $(snapshot_taps); do
        ip link del "$t" 2>/dev/null || true
    done
    log "cleaned up bridge and taps"
}
trap cleanup EXIT

mkdir -p "$REPORT_DIR" "$WORK_DIR"

setup_hugepages
build_harness
bridge_up

run_one_scenario() {
    case "$1" in
        bidirectional_unicast) scenario_bidi "$1" dpdk dpdk ;;
        dpdk_to_udp)          scenario_bidi "$1" dpdk udp ;;
        udp_to_dpdk)          scenario_bidi "$1" udp dpdk ;;
        reconnect)            scenario_reconnect ;;
        multi_endpoint)       scenario_multi_endpoint ;;
        loss_recovery)        scenario_bidi "$1" dpdk dpdk ;;
        restart)              scenario_restart ;;
        *) die "unknown scenario $1" ;;
    esac
}

if [[ -n "$SCENARIO" ]]; then
    run_one_scenario "$SCENARIO"
else
    for s in $ALL_SCENARIOS; do
        log "=== scenario: $s ==="
        run_one_scenario "$s"
    done
fi

CLEANUP_ON_EXIT=0
cleanup
log "ALL VDEV SCENARIOS PASSED"
