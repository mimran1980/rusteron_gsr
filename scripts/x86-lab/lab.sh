#!/usr/bin/env bash
# rusteron's x86-64 lab: an Intel VM and an AMD VM in Azure resource group rusteron-lab,
# which exists only while this script runs. It creates both, copies this checkout, main's
# tree and the harness, runs harness/vm.sh's phases on both at once, copies the results to
# target/x86lab/results/<stamp>/<node>/after-<phase> after each phase, and deletes the group
# on any exit. Runs on macOS (bsdtar, caffeinate) with the az CLI logged in and
# ~/.ssh/id_rsa.pub as the VMs' key; summarise a bench.csv with analyse.py. It refuses to
# start unless more than 10 USD of free credit is left.
#
#   scripts/x86-lab/lab.sh                            every phase
#   LAB_PHASES="bootstrap build bench" scripts/x86-lab/lab.sh
#   LAB_ARMS="main impr" (only these harness arms), LAB_TESTS=workspace (only the
#   workspace tests) and LAB_EXTRAS=samples (also build the Java and Rust samples) pass
#   through to the VMs. LAB_NODES="name:region:size ..." replaces the two default nodes;
#   LAB_PAIR=1 puts each region's nodes in a placement group and, after the phases, runs
#   the cross-host UDP bench from each region's first node to its second
set -uo pipefail
export PYTHONWARNINGS=ignore::SyntaxWarning COPYFILE_DISABLE=1

# az 2.90 on Python 3.14 sometimes dies importing `requests` on two threads at once
# (`_DeadlockError`) before it sends anything; the same call then succeeds. Only that
# failure is retried. By path, not `command az`: bash 3.2 execs a command run through
# `command` in place of a background subshell.
az_bin=$(type -P az) || { echo "az is not on PATH" >&2; exit 1; }
az() {
    local err rc attempt
    for attempt in 1 2 3 4 5; do
        rc=0
        # stdout passes through to the caller; stderr is kept to look for the deadlock
        { err=$("$az_bin" "$@" 2>&1 1>&3 3>&-); } 3>&1 || rc=$?
        if [[ -n $err ]]; then printf '%s\n' "$err" >&2; fi
        if ((rc == 0)) || [[ $err != *_DeadlockError* ]]; then return "$rc"; fi
        echo "az: import deadlock (attempt $attempt), retrying: az $1 ${2:-}" >&2
    done
    return "$rc"
}

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
group=rusteron-lab
user=$(id -un)
out=$repo/target/x86lab/results/$(date +%Y%m%d-%H%M%S)
# main's tree, for the arms that A/B against it
main_tree=$repo/target/x86lab/rusteron-main
known=$out/known_hosts
read -ra nodes <<<"${LAB_NODES:-intel:northcentralus:Standard_D4s_v6 amd:koreacentral:Standard_F4as_v6}"
read -ra phases <<<"${LAB_PHASES:-bootstrap build bench test}"
ssh_opts=(-o BatchMode=yes -o StrictHostKeyChecking=accept-new -o "UserKnownHostsFile=$known"
    -o ConnectTimeout=15 -o ServerAliveInterval=30 -o ServerAliveCountMax=10)
created=0
before=?

log() { echo "[$(date +%T)] $*"; }

# the estimated free credit left on the first billing account's first billing profile, in USD
credit_left() {
    local api=https://management.azure.com/providers/Microsoft.Billing/billingAccounts acc profile
    acc=$(az rest --method get --url "$api?api-version=2024-04-01" --query "value[0].name" -o tsv) || return
    profile=$(az rest --method get --url "$api/$acc/billingProfiles?api-version=2024-04-01" --query "value[0].name" -o tsv) || return
    az rest --method get -o tsv --query properties.balanceSummary.estimatedBalance.value \
        --url "$api/$acc/billingProfiles/$profile/providers/Microsoft.Consumption/credits/balanceSummary?api-version=2023-05-01"
}

# seconds a phase may run on a VM
limit() {
    case $1 in
        bootstrap) echo 1800 ;;
        build) echo 4800 ;;
        bench) echo 3600 ;;
        abudp) echo 2400 ;;
        bench-pinned | bench-isolated | bench-huge | bench-1g | k8s | k8s-cpu | bench8 | archive8 | bench8-isolated) echo 3600 ;;
        bench8-tuned | bench8-tuned-nomit) echo 3600 ;;
        k8s8) echo 5400 ;;
        isolate | isolate8 | tune8 | tune8-nomit | k3s-down) echo 600 ;;
        test) echo 5400 ;;
    esac
}

teardown() {
    local rc=$? id attempt
    trap - EXIT HUP INT TERM
    if ((created)); then
        kill $(jobs -p) 2>/dev/null
        log "teardown: deleting $group"
        for _ in 1 2 3; do
            if az group delete -n "$group" --yes; then break; fi
            sleep 20
        done
        # the network watchers Azure added for our networks, and their group if that empties it;
        # a watcher can show up in listings late, so look three times
        for attempt in 1 2 3; do
            id=$(az resource list -g NetworkWatcherRG --query "[?type=='Microsoft.Network/networkWatchers' && \
                (location=='northcentralus' || location=='koreacentral')].id" -o tsv 2>/dev/null)
            if [[ -n $id ]]; then az resource delete --ids $id -o none; fi
            if ((attempt < 3)); then sleep 20; fi
        done
        if [[ $(az group exists -n NetworkWatcherRG) == true &&
            $(az resource list -g NetworkWatcherRG --query 'length(@)' -o tsv) == 0 ]]; then
            az group delete -n NetworkWatcherRG --yes
        fi
        log "group $group exists: $(az group exists -n "$group")"
        log "resources in the subscription: $(az resource list --query 'length(@)' -o tsv) (before up: $before)"
        az resource list --query '[].[resourceGroup, type, name, location]' -o tsv
    fi
    exit "$rc"
}

# one network per region, so the nodes of a region share a subnet; with LAB_PAIR also a
# proximity placement group, so a pair sits as close together as Azure allows
create_network() {
    local region=$1
    az network nsg create -g "$group" -n "$region-nsg" -l "$region" -o none &&
        az network nsg rule create -g "$group" --nsg-name "$region-nsg" -n ssh-from-operator --priority 100 \
            --source-address-prefixes "$mine" --destination-port-ranges 22 --protocol Tcp --access Allow -o none &&
        az network vnet create -g "$group" -n "$region-vnet" -l "$region" --address-prefixes 10.60.0.0/16 \
            --subnet-name s --subnet-prefixes 10.60.0.0/24 --network-security-group "$region-nsg" -o none || return 1
    if [[ -n ${LAB_PAIR:-} ]]; then
        az ppg create -g "$group" -n "$region-ppg" -l "$region" -t Standard -o none || return 1
    fi
}

create_node() {
    local name=$1 region=$2 size=$3 ppg=()
    if [[ -n ${LAB_PAIR:-} ]]; then ppg=(--ppg "$region-ppg"); fi
    az vm create -g "$group" -n "lab-$name" -l "$region" --size "$size" \
        --image Debian:debian-13:13-gen2:latest --admin-username "$user" --ssh-key-values "$key" \
        --vnet-name "$region-vnet" --subnet s --nsg "" --public-ip-address "$name-ip" --public-ip-sku Standard \
        --accelerated-networking true --os-disk-size-gb 64 --storage-sku Premium_LRS ${ppg[@]+"${ppg[@]}"} \
        --disk-controller-type NVMe --os-disk-delete-option Delete --nic-delete-option Delete -o none
}

# pair_bench <ping node> <pong node>: cross-host UDP, driven from the ping node over ssh
pair_bench() {
    local a=$1 b=$2 a_ip b_ip b_private pub rc=0
    a_ip=$(az vm show -d -g "$group" -n "lab-$a" --query publicIps -o tsv)
    b_ip=$(az vm show -d -g "$group" -n "lab-$b" --query publicIps -o tsv)
    b_private=$(az vm show -d -g "$group" -n "lab-$b" --query privateIps -o tsv)
    ssh "${ssh_opts[@]}" "$user@$a_ip" 'test -f ~/.ssh/id_ed25519 || ssh-keygen -q -t ed25519 -N "" -f ~/.ssh/id_ed25519'
    pub=$(ssh "${ssh_opts[@]}" "$user@$a_ip" cat .ssh/id_ed25519.pub)
    ssh "${ssh_opts[@]}" "$user@$b_ip" "echo '$pub' >>~/.ssh/authorized_keys"
    log "$a -> $b: bench-xhost"
    ssh "${ssh_opts[@]}" "$user@$a_ip" \
        "LAB_PEER_IP=$b_private BENCH3_REPS='${BENCH3_REPS:-}' timeout 3600 /srv/x86lab/harness/vm.sh bench-xhost" \
        >"$out/$a/bench-xhost.log" 2>&1 || rc=$?
    fetch "$a_ip" "$out/$a/after-bench-xhost"
    log "$a -> $b: bench-xhost exit $rc"
    return "$rc"
}

# main at its own submodule commits, extracted once into target/
make_main_tree() {
    local sub
    if [[ -d $main_tree ]]; then return; fi
    mkdir -p "$main_tree"
    git -C "$repo" archive main | tar -x -C "$main_tree"
    for sub in rusteron-client/aeron rusteron-archive/aeron rusteron-media-driver/aeron; do
        git -C "$repo/$sub" archive "$(git -C "$repo" rev-parse "main:$sub")" | tar -x -C "$main_tree/$sub"
    done
}

# what git tracks in this checkout (with the submodules), main's tree and the harness;
# no build output and no macOS prebuilt libraries
sync_to() {
    local ip=$1
    (cd "$repo" && git ls-files --recurse-submodules -z |
        tar --null -T - --no-xattrs --no-mac-metadata -czf - |
        ssh "${ssh_opts[@]}" "$user@$ip" 'mkdir -p /srv/x86lab/rusteron && tar -xzf - -C /srv/x86lab/rusteron') &&
        tar -C "$here" --no-xattrs --no-mac-metadata --exclude target -czf - harness |
        ssh "${ssh_opts[@]}" "$user@$ip" 'tar -xzf - -C /srv/x86lab' &&
        tar -C "$(dirname "$main_tree")" --no-xattrs --no-mac-metadata -czf - rusteron-main |
        ssh "${ssh_opts[@]}" "$user@$ip" 'tar -xzf - -C /srv/x86lab'
}

fetch() {
    local ip=$1 dest=$2
    mkdir -p "$dest"
    ssh "${ssh_opts[@]}" "$user@$ip" 'tar -C /srv/x86lab -czf - results' | tar -xzf - -C "$dest"
}

run_node() {
    local name=$1 ip=$2 dest=$out/$1 rc phase
    mkdir -p "$dest"
    for _ in $(seq 60); do
        if ssh "${ssh_opts[@]}" "$user@$ip" true 2>/dev/null; then break; fi
        sleep 5
    done
    ssh "${ssh_opts[@]}" "$user@$ip" 'sudo install -d -o "$(id -un)" -g "$(id -gn)" /srv/x86lab' || return 1
    if ! sync_to "$ip" >"$dest/sync.log" 2>&1; then
        log "$name: sync failed"
        return 1
    fi
    for phase in "${phases[@]}"; do
        log "$name: $phase"
        rc=0
        ssh "${ssh_opts[@]}" "$user@$ip" \
            "LAB_ARMS='${LAB_ARMS:-}' LAB_TESTS='${LAB_TESTS:-}' LAB_EXTRAS='${LAB_EXTRAS:-}' timeout $(limit "$phase") /srv/x86lab/harness/vm.sh $phase" \
            >"$dest/$phase.log" 2>&1 || rc=$?
        if [[ $phase == isolate* || $phase == tune8* ]]; then
            # the VM reboots a few seconds after the phase returns
            sleep 45
            for _ in $(seq 60); do
                if ssh "${ssh_opts[@]}" "$user@$ip" true 2>/dev/null; then break; fi
                sleep 5
            done
        fi
        fetch "$ip" "$dest/after-$phase"
        log "$name: $phase exit $rc"
        if ((rc != 0)) && [[ $phase == bootstrap || $phase == build ]]; then return "$rc"; fi
    done
}

main() {
    mkdir -p "$out"
    # only on free credit: the subscription has no spending limit, so past the credit a card pays
    local credit
    credit=$(credit_left)
    if ! [[ $credit =~ ^[0-9]+(\.[0-9]+)?$ ]] || ! awk -v c="$credit" 'BEGIN { exit !(c > 10) }'; then
        echo "free credit left: '${credit}' USD, at or below the 10 USD margin or unreadable: not creating anything" >&2
        exit 1
    fi
    log "free credit left: $credit USD"
    if [[ $(az group exists -n "$group") != false ]]; then
        echo "$group already exists: not created by this run, so not touched" >&2
        exit 1
    fi
    before=$(az resource list --query 'length(@)' -o tsv)
    { git -C "$repo" rev-parse HEAD main; git -C "$repo" status --short; } >"$out/shas.txt"
    key=$HOME/.ssh/id_rsa.pub
    mine=$(curl -fsS https://api.ipify.org)/32
    # the Mac must not sleep while the lab runs
    if command -v caffeinate >/dev/null; then caffeinate -i -w $$ & fi
    make_main_tree

    trap teardown EXIT HUP INT TERM
    created=1
    log "up: $group"
    az group create -n "$group" -l northcentralus -o none || exit 1
    local spec name region size pids=() failed=0 p regions=" "
    for spec in "${nodes[@]}"; do
        IFS=: read -r name region size <<<"$spec"
        if [[ $regions != *" $region "* ]]; then
            regions+="$region "
            create_network "$region" >"$out/network-$region.log" 2>&1 || { log "network $region failed"; exit 1; }
        fi
    done
    for spec in "${nodes[@]}"; do
        IFS=: read -r name region size <<<"$spec"
        create_node "$name" "$region" "$size" >"$out/create-$name.log" 2>&1 &
        pids+=($!)
    done
    for p in "${pids[@]}"; do wait "$p" || failed=1; done
    if ((failed)); then
        log "a VM failed to create: see $out/create-*.log"
        exit 1
    fi

    pids=()
    for spec in "${nodes[@]}"; do
        IFS=: read -r name region size <<<"$spec"
        local ip
        ip=$(az vm show -d -g "$group" -n "lab-$name" --query publicIps -o tsv)
        log "$name ($size, $region): $ip"
        run_node "$name" "$ip" &
        pids+=($!)
    done
    for p in "${pids[@]}"; do wait "$p" || failed=1; done
    if [[ -n ${LAB_PAIR:-} ]] && ((failed == 0)); then
        # in each region, the first node pings and the second pongs
        pids=()
        for region in $regions; do
            local pair=()
            for spec in "${nodes[@]}"; do
                IFS=: read -r name r size <<<"$spec"
                if [[ $r == "$region" ]]; then pair+=("$name"); fi
            done
            if ((${#pair[@]} == 2)); then
                pair_bench "${pair[0]}" "${pair[1]}" &
                pids+=($!)
            fi
        done
        for p in "${pids[@]}"; do wait "$p" || failed=1; done
    fi
    log "lab run finished (failed=$failed); results in $out"
    exit "$failed"
}

main
