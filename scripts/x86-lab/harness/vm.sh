#!/usr/bin/env bash
# Runs on a lab VM: vm.sh bootstrap|build|bench|test. Everything lives in /srv/x86lab;
# results go to /srv/x86lab/results, which the Mac copies after each phase.
set -euo pipefail

lab=${X86LAB:-/srv/x86lab}
shm=${SHM:-/dev/shm}
res=$lab/results
bin=$lab/bin
mkdir -p "$res" "$bin"
export RUSTUP_TOOLCHAIN=1.95.0 CARGO_TERM_COLOR=never PATH=$HOME/.cargo/bin:$PATH
host=$(hostname)
native="-C target-cpu=native"

log() { echo "[$(date +%T)] $*"; }

bootstrap() {
    sudo apt-get update -qq
    sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq build-essential cmake clang libclang-dev \
        pkg-config libbsd-dev uuid-dev zlib1g-dev libssl-dev default-jdk-headless curl util-linux >/dev/null
    # rustfmt: the build scripts format the generated bindings and fail without it
    curl -sSf https://sh.rustup.rs | sh -s -- -y -q --profile minimal --default-toolchain 1.95.0 --component rustfmt
    # Linux silently caps SO_RCVBUF/SO_SNDBUF at these
    sudo sysctl -q -w net.core.rmem_max=16777216 net.core.wmem_max=16777216
    {
        uname -a
        lscpu
        lscpu -e
        free -g
        echo "thp: $(cat /sys/kernel/mm/transparent_hugepage/enabled) shmem: $(cat /sys/kernel/mm/transparent_hugepage/shmem_enabled)"
        findmnt /dev/shm
        rustc --version
        gcc --version | head -1
        clang --version | head -1
        java -version 2>&1 | head -1
        cmake --version | head -1
    } >"$res/system.txt" 2>&1
    log "bootstrap done"
}

# One harness build: arm <name> <rusteron root> <features> <rustflags> [env...]. Arms with the
# same rustflags share a target dir; each arm's binaries are copied out before the next build.
# LAB_ARMS, when set, names the only arms to build.
arm() {
    local name=$1 root=$2 features=$3 rustflags=$4
    shift 4
    if [[ -n ${LAB_ARMS:-} && " $LAB_ARMS " != *" $name "* ]]; then return; fi
    local dir=$lab/arms/$name target
    target=$lab/target-$(tr -c 'a-z0-9\n' '_' <<<"$rustflags")
    mkdir -p "$dir" "$bin/$name"
    sed "s|@RUSTERON@|$root|" "$lab/harness/Cargo.toml.in" >"$dir/Cargo.toml"
    ln -sfn "$lab/harness/src" "$dir/src"
    log "build $name"
    (cd "$dir" && env "$@" RUSTFLAGS="$rustflags" CARGO_TARGET_DIR="$target" \
        cargo build --release --features "$features" --bins) >"$res/build-$name.log" 2>&1
    for b in rtt tput pspoll; do
        if [[ -e $target/release/$b && $target/release/$b -nt $dir/Cargo.toml ]]; then cp "$target/release/$b" "$bin/$name/"; fi
    done
    cp "$dir/Cargo.lock" "$res/Cargo.lock-$name"
    find "$target/release/build" -name CMakeCache.txt -newer "$dir/Cargo.toml" \
        -exec grep -H 'CMAKE_C_FLAGS_RELEASE:' {} + >"$res/cflags-$name.txt" || true
    # a dynamic build loads libaeron from its build's out dir
    if [[ $features != *static* ]]; then
        find "$target/release/build" -name 'libaeron.so*' -newer "$dir/Cargo.toml" -exec cp -a {} "$bin/$name/" \;
    fi
}

# Aeron's Java samples and rusteron's ports of them, for the Java comparison
build_samples() {
    log "build Java samples"
    (cd "$lab/rusteron/rusteron-archive/aeron" && ./gradlew -q :aeron-all:jar :aeron-samples:jar) >"$res/build-java.log" 2>&1
    log "build rust examples"
    (cd "$lab/rusteron" && RUSTFLAGS="$native" CARGO_TARGET_DIR="$lab/target-examples" \
        cargo build --release -p rusteron-client --features "examples static" \
        --example embedded_ping_pong --example embedded_exclusive_ipc_throughput) >"$res/build-examples.log" 2>&1
    mkdir -p "$bin/examples"
    cp "$lab/target-examples/release/examples/embedded_ping_pong" \
        "$lab/target-examples/release/examples/embedded_exclusive_ipc_throughput" "$bin/examples/"
}

build() {
    arm main "$lab/rusteron-main" static "$native"
    arm impr "$lab/rusteron" static "$native"
    arm impr-c-x86-64 "$lab/rusteron" static "$native" RUSTERON_C_MARCH=x86-64
    arm impr-c-x86-64-v3 "$lab/rusteron" static "$native" RUSTERON_C_MARCH=x86-64-v3
    arm impr-dynamic "$lab/rusteron" "" "$native"
    arm impr-rust-x86-64 "$lab/rusteron" static "-C target-cpu=x86-64"
    arm impr-ps "$lab/rusteron" "static archive" "$native"
    # one driver for every run, so the arms differ only in the client
    log "build media_driver"
    (cd "$lab/rusteron" && RUSTFLAGS="$native" CARGO_TARGET_DIR="$lab/target-driver" \
        cargo build --release -p rusteron-media-driver --features static --bin media_driver) >"$res/build-driver.log" 2>&1
    cp "$lab/target-driver/release/media_driver" "$bin/"
    if [[ ${LAB_EXTRAS:-} == *samples* ]]; then build_samples; fi
    ls -la "$bin"/* >"$res/binaries.txt"
    log "build done"
}

# ping, pong and housekeeping CPUs for each layout. split: ping and pong on separate
# cores; smt: on the two threads of one core (SMT machines only); unpinned: no pinning.
topology() {
    local sib s
    sib=$(cat "${CPU3_SIBLINGS:-/sys/devices/system/cpu/cpu3/topology/thread_siblings_list}")
    if [[ $sib == 3 ]]; then
        smt=0
        layout_split=(2 3 0,1)
    else
        smt=1
        s=$(tr ',-' '\n\n' <<<"$sib" | grep -vx 3 | head -1)
        local others=()
        for c in 0 1 2; do if [[ $c != "$s" ]]; then others+=("$c"); fi; done
        # ping off cpu0, which takes the VM's interrupts
        layout_split=("${others[1]}" 3 "${others[0]},$s")
        layout_smt=("$s" 3 "${others[0]},${others[1]}")
    fi
    layout_unpinned=(- - 0-3)
    echo "smt=$smt split=${layout_split[*]} smt_layout=${layout_smt[*]:-} siblings_of_3=$sib" >"$res/layouts.txt"
}

use_layout() {
    layout=$1
    local l
    case $1 in
        split) l=("${layout_split[@]}") ;;
        smt) l=("${layout_smt[@]}") ;;
        unpinned) l=("${layout_unpinned[@]}") ;;
    esac
    ping=${l[0]} pong=${l[1]} hk=${l[2]}
}

runs=0
driver_start() {
    runs=$((runs + 1))
    run_dir=${run_base:-$shm}/x86lab-$runs
    env AERON_DIR="$run_dir" AERON_DIR_DELETE_ON_START=true AERON_DIR_DELETE_ON_SHUTDOWN=true "$@" \
        taskset -c "$hk" "$bin/media_driver" >"$res/driver-last.log" 2>&1 &
    driver=$!
    for _ in $(seq 100); do
        if [[ -e $run_dir/cnc.dat ]]; then break; fi
        sleep 0.1
    done
    sleep 0.5
}

driver_stop() {
    kill -INT "$driver" 2>/dev/null || true
    wait "$driver" 2>/dev/null || true
}

IPC_N=2000000 IPC_W=200000 UDP_N=300000 UDP_W=50000

# run <group> <label> <arm> <ipc|udp|tput> [env for driver and client...]
run() {
    local group=$1 label=$2 arm=$3 test=$4
    shift 4
    local cmd line
    case $test in
        ipc) cmd=(rtt ipc "$IPC_N" "$IPC_W" "$ping" "$pong") ;;
        udp) cmd=(rtt udp "$UDP_N" "$UDP_W" "$ping" "$pong") ;;
        tput) cmd=(tput 5 "$ping" "$pong") ;;
    esac
    driver_start "$@"
    line=$(env "$@" AERON_DIR="$run_dir" LABEL="$label" LD_LIBRARY_PATH="$bin/$arm" \
        taskset -c "$hk" timeout 300 "$bin/$arm/${cmd[0]}" "${cmd[@]:1}" 2>>"$res/client-errors.log") || line="error,$label,$test"
    driver_stop
    echo "$host,$group,$layout,$rep,$line" | tee -a "$res/bench.csv"
}

shm_huge() {
    sudo mount -o remount,huge="$1" "$shm"
}

# rotate <n> <items...>: the items rotated left by n, so each rep starts with a different arm
rotate() {
    local n=$1
    shift
    local items=("$@") k
    k=$((n % ${#items[@]}))
    echo "${items[@]:k}" "${items[@]:0:k}"
}

bench() {
    topology
    local udp_base=(AERON_SENDER_IDLE_STRATEGY=noop AERON_RECEIVER_IDLE_STRATEGY=noop)
    # the client's conductor and the driver's settings, for the record
    use_layout split
    driver_start AERON_PRINT_CONFIGURATION=true "${udp_base[@]}"
    sleep 1
    driver_stop
    cp "$res/driver-last.log" "$res/driver-config.txt"

    log "A/B: main against improvements, with an A/A pair"
    for rep in 1 2 3 4 5 6; do
        for a in $(rotate "$rep" main impr impr-aa); do
            local binary=${a%-aa}
            for t in ipc udp tput; do run ab "$a" "$binary" "$t" "${udp_base[@]}"; done
        done
    done

    log "build variants"
    for rep in 1 2 3 4 5; do
        for a in $(rotate "$rep" impr impr-c-x86-64 impr-c-x86-64-v3 impr-dynamic impr-rust-x86-64); do
            for t in ipc tput; do run build "$a" "$a" "$t"; done
        done
    done

    log "IPC settings, one at a time from the defaults"
    local knobs=(base nonsparse pretouch term1m shmhuge combo unpinned)
    if ((smt)); then knobs+=(smt); fi
    for rep in 1 2 3 4 5; do
        for k in $(rotate "$rep" "${knobs[@]}"); do
            local envs=()
            use_layout split
            case $k in
                nonsparse) envs=(AERON_TERM_BUFFER_SPARSE_FILE=false) ;;
                pretouch) envs=(AERON_TERM_BUFFER_SPARSE_FILE=false AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true) ;;
                term1m) envs=(AERON_IPC_TERM_BUFFER_LENGTH=1048576) ;;
                shmhuge) shm_huge always ;;
                combo) envs=(AERON_TERM_BUFFER_SPARSE_FILE=false AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true) && shm_huge always ;;
                unpinned) use_layout unpinned ;;
                smt) use_layout smt ;;
            esac
            for t in ipc tput; do run ipc-knobs "$k" impr "$t" ${envs[@]+"${envs[@]}"}; done
            shm_huge never
        done
    done

    log "UDP driver threading and idle strategies"
    local udp_knobs=(dedicated-noop dedicated-backoff dedicated-spin shared-network-noop shared-noop)
    if ((smt)); then udp_knobs+=(dedicated-noop-smt shared-network-noop-smt); fi
    for rep in 1 2 3 4 5; do
        for k in $(rotate "$rep" "${udp_knobs[@]}"); do
            local envs=()
            use_layout split
            case $k in
                dedicated-noop) envs=("${udp_base[@]}") ;;
                dedicated-backoff) envs=() ;;
                dedicated-spin) envs=(AERON_SENDER_IDLE_STRATEGY=spin AERON_RECEIVER_IDLE_STRATEGY=spin) ;;
                shared-network-noop) envs=(AERON_THREADING_MODE=SHARED_NETWORK AERON_SHAREDNETWORK_IDLE_STRATEGY=noop) ;;
                shared-noop) envs=(AERON_THREADING_MODE=SHARED AERON_SHARED_IDLE_STRATEGY=noop) ;;
                dedicated-noop-smt) envs=("${udp_base[@]}") && use_layout smt ;;
                shared-network-noop-smt) envs=(AERON_THREADING_MODE=SHARED_NETWORK AERON_SHAREDNETWORK_IDLE_STRATEGY=noop) && use_layout smt ;;
            esac
            run udp-knobs "$k" impr udp ${envs[@]+"${envs[@]}"}
        done
    done

    log "persistent subscription poll cost"
    use_layout unpinned
    for rep in 1 2; do
        for mode in thread invoker; do
            env PSPOLL_DIR="$shm/x86lab-ps" LABEL="ps-$rep" timeout 900 "$bin/impr-ps/pspoll" "$mode" fixed 1 10 100 \
                2>>"$res/pspoll-errors.log" | grep '^pspoll' | sed "s/^/$host,ps,unpinned,$rep,/" | tee -a "$res/bench.csv" || true
        done
    done
    log "bench done"
}

# the UDP A/B again with more reps: as in bench's ab group, then with pre-touched logs so
# first-touch page faults do not add noise
abudp() {
    topology
    use_layout split
    local udp_base=(AERON_SENDER_IDLE_STRATEGY=noop AERON_RECEIVER_IDLE_STRATEGY=noop)
    local pretouch=(AERON_TERM_BUFFER_SPARSE_FILE=false AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true)
    log "UDP A/B, 12 reps"
    for rep in $(seq 1 12); do
        for a in $(rotate "$rep" main impr impr-aa); do
            run ab-udp "$a" "${a%-aa}" udp "${udp_base[@]}"
            run ab-udp-pretouch "$a" "${a%-aa}" udp "${udp_base[@]}" "${pretouch[@]}"
        done
    done
    log "abudp done"
}

# pin_rust <pid> <thread>=<cpu>...: "main" is the process's first thread, others match by name
pin_rust() {
    local pid=$1 spec name cpu tid t
    shift
    for spec in "$@"; do
        name=${spec%=*} cpu=${spec#*=} tid=
        for _ in $(seq 200); do
            if [[ $name == main ]]; then tid=$pid; else
                for t in /proc/"$pid"/task/*; do
                    if [[ $(cat "$t/comm" 2>/dev/null) == "$name" ]]; then tid=${t##*/}; fi
                done
            fi
            if [[ -n $tid ]]; then break; fi
            sleep 0.05
        done
        if [[ -z $tid ]]; then return 1; fi
        taskset -pc "$cpu" "$tid" >/dev/null || return 1
    done
}

# pin_java <pid> <thread name prefix>=<cpu>...: jcmd maps Java thread names to native ids
pin_java() {
    local pid=$1 spec name cpu nid dump
    shift
    for spec in "$@"; do
        name=${spec%=*} cpu=${spec#*=} nid=
        for _ in $(seq 60); do
            dump=$(jcmd "$pid" Thread.print 2>/dev/null) || true
            nid=$(awk -v n="\"$name" 'index($0, n) == 1 { for (i = 1; i <= NF; i++) if ($i ~ /^nid=/) { sub("nid=", "", $i); print $i; exit } }' <<<"$dump")
            if [[ -n $nid ]]; then break; fi
            sleep 0.2
        done
        if [[ -z $nid ]]; then return 1; fi
        taskset -pc "$cpu" "$((nid))" >/dev/null || return 1
    done
}

# run_sample <group> <label> <java-pp|rust-pp|java-tput|rust-tput> [env...]: one sample with its
# two hot threads pinned to $ping and $pong; the Rust ones use the external driver, the Java ones
# embed Aeron's Java driver
run_sample() {
    local group=$1 label=$2 kind=$3 out=$res/sample-last.out top pid row pinned=yes skip=4
    shift 3
    local jars=$lab/rusteron/rusteron-archive/aeron
    local java=(java -cp "$jars/aeron-all/build/libs/aeron-all-1.52.2.jar:$jars/aeron-samples/build/libs/aeron-samples-1.52.2.jar"
        --add-opens java.base/jdk.internal.misc=ALL-UNNAMED -Dagrona.disable.bounds.checks=true
        -Daeron.dir.delete.on.start=true -Daeron.dir.delete.on.shutdown=true -Daeron.term.buffer.sparse.file=false
        -Daeron.pre.touch.mapped.memory=true -Daeron.sample.messageLength=32
        -Daeron.sample.idleStrategy=org.agrona.concurrent.NoOpIdleStrategy)
    if [[ $kind == rust-* ]]; then
        driver_start "$@"
    else
        runs=$((runs + 1))
        run_dir=${run_base:-$shm}/x86lab-$runs
    fi
    case $kind in
        java-pp)
            printf 'n\n' | taskset -c "$hk" timeout 180 "${java[@]}" -Daeron.dir="$run_dir" \
                -Daeron.sample.messages=1000000 -Daeron.sample.warmup.iterations=30 \
                -Daeron.sample.exclusive.publications=true \
                '-Daeron.sample.ping.channel=aeron:udp?endpoint=localhost:20123' \
                '-Daeron.sample.pong.channel=aeron:udp?endpoint=localhost:20124' \
                io.aeron.samples.EmbeddedPingPong >"$out" 2>&1 &
            ;;
        java-tput)
            taskset -c "$hk" timeout -s INT 12 "${java[@]}" -Daeron.dir="$run_dir" \
                io.aeron.samples.EmbeddedExclusiveIpcThroughput </dev/null >"$out" 2>&1 &
            ;;
        rust-pp)
            env "$@" AERON_DIR="$run_dir" taskset -c "$hk" timeout 180 "$bin/examples/embedded_ping_pong" >"$out" 2>&1 &
            ;;
        rust-tput)
            skip=2
            env "$@" AERON_DIR="$run_dir" taskset -c "$hk" timeout -s INT 12 \
                "$bin/examples/embedded_exclusive_ipc_throughput" </dev/null >"$out" 2>&1 &
            ;;
    esac
    top=$!
    pid=
    for _ in $(seq 100); do
        pid=$(pgrep -n -P "$top") && break
        sleep 0.05
    done
    case $kind in
        java-pp) pin_java "$pid" main="$ping" Thread-="$pong" || pinned=no ;;
        java-tput) pin_java "$pid" publisher="$ping" subscriber="$pong" || pinned=no ;;
        rust-pp) pin_rust "$pid" main="$ping" pong="$pong" || pinned=no ;;
        rust-tput) pin_rust "$pid" main="$ping" subscriber="$pong" || pinned=no ;;
    esac
    wait "$top" || true
    if [[ $kind == rust-* ]]; then driver_stop; fi
    cp "$out" "$res/samples/$group-$label-$rep.out"
    row=$(python3 "$lab/harness/parse_samples.py" "$kind" "$out" "$label" "$skip" 2>>"$res/client-errors.log") || row="error,$label,$kind"
    if [[ $pinned == no ]]; then row=${row/,$label,/,$label-unpinned,}; fi
    echo "$host,$group,$layout,$rep,$row" | tee -a "$res/bench.csv"
}

# can the driver start with its directory on hugetlbfs?
hugetlbfs_works() {
    local ok=no
    run_base=/mnt/huge driver_start AERON_FILE_PAGE_SIZE=2097152 AERON_TERM_BUFFER_SPARSE_FILE=false
    sleep 1
    if kill -0 "$driver" 2>/dev/null && [[ -e $run_dir/cnc.dat ]]; then ok=yes; fi
    driver_stop
    cp "$res/driver-last.log" "$res/hugetlbfs-probe-$state.log"
    echo "hugetlbfs driver start: $ok" >>"$res/state-$state.txt"
    [[ $ok == yes ]]
}

# The Java comparison and the huge-page modes, with the hot threads on CPUs 2 and 3 and
# everything else on 0 and 1; <state> says whether 2 and 3 are isolated (after `isolate`)
bench3() {
    state=$1
    ping=2 pong=3 hk=0,1 layout=hot23
    mkdir -p "$res/samples"
    {
        echo "state=$state"
        cat /proc/cmdline || true
        echo "isolated=$(cat /sys/devices/system/cpu/isolated 2>/dev/null) nohz_full=$(cat /sys/devices/system/cpu/nohz_full 2>/dev/null)"
    } >"$res/state-$state.txt"
    local tuned=(AERON_TERM_BUFFER_SPARSE_FILE=false AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true
        AERON_SENDER_IDLE_STRATEGY=noop AERON_RECEIVER_IDLE_STRATEGY=noop)
    # explicit 2 MiB huge pages for a hugetlbfs AERON_DIR, whose files must be sized in huge pages
    sudo sysctl -q -w vm.nr_hugepages=1536
    sudo mkdir -p /mnt/huge
    # size= gives the mount a capacity, without which Aeron's storage check sees no usable space
    mountpoint -q /mnt/huge || sudo mount -t hugetlbfs -o "pagesize=2M,size=2G,uid=$(id -u),gid=$(id -g)" none /mnt/huge
    grep -E 'HugePages_(Total|Free)' /proc/meminfo >>"$res/state-$state.txt" || true
    local modes=(off shm hugetlbfs)
    if ! hugetlbfs_works; then modes=(off shm); fi
    log "bench3 $state: modes ${modes[*]}"
    for rep in $(seq "${BENCH3_REPS:-5}"); do
        for mode in $(rotate "$rep" "${modes[@]}"); do
            local g=$state-$mode extra=()
            run_base=
            case $mode in
                shm) shm_huge always ;;
                hugetlbfs) run_base=/mnt/huge extra=(AERON_FILE_PAGE_SIZE=2097152) ;;
            esac
            for t in ipc udp tput; do run "$g" "h-$t" impr "$t" "${tuned[@]}" ${extra[@]+"${extra[@]}"}; done
            if [[ $mode != hugetlbfs && ${BENCH3_SAMPLES:-1} == 1 ]]; then
                run_sample "$g" java-tput java-tput
                run_sample "$g" rust-tput rust-tput "${tuned[@]}"
                run_sample "$g" java-pp java-pp
                run_sample "$g" rust-pp rust-pp "${tuned[@]}"
            fi
            shm_huge never
        done
    done
    run_base=
    log "bench3 $state done"
}

# isolate CPUs 2 and 3 from the scheduler, timer ticks, RCU callbacks and IRQs, then reboot
isolate() {
    sudo mkdir -p /etc/default/grub.d
    printf '%s\n' 'GRUB_CMDLINE_LINUX_DEFAULT="$GRUB_CMDLINE_LINUX_DEFAULT isolcpus=nohz,domain,managed_irq,2,3 nohz_full=2,3 rcu_nocbs=2,3 irqaffinity=0,1"' |
        sudo tee /etc/default/grub.d/99-rusteron-isolation.cfg >/dev/null
    sudo update-grub >"$res/update-grub.log" 2>&1
    grep -c 'isolcpus=' /boot/grub/grub.cfg >>"$res/update-grub.log" 2>&1 || true
    sudo systemd-run --on-active=3 /bin/systemctl reboot >/dev/null
    log "rebooting into isolation"
}

# 2 MiB against 1 GiB huge pages for AERON_DIR (hugetlbfs), IPC only: with 1 GiB pages every
# Aeron file rounds up to a whole page, so a UDP ping-pong would need about 10 GiB of them
bench_1g() {
    state=pinned
    ping=2 pong=3 hk=0,1 layout=hot23
    sudo sysctl -q -w vm.nr_hugepages=1024
    echo 8 | sudo tee /sys/kernel/mm/hugepages/hugepages-1048576kB/nr_hugepages >/dev/null
    sudo mkdir -p /mnt/huge2m /mnt/huge1g
    mountpoint -q /mnt/huge2m || sudo mount -t hugetlbfs -o "pagesize=2M,size=2G,uid=$(id -u),gid=$(id -g)" none /mnt/huge2m
    mountpoint -q /mnt/huge1g || sudo mount -t hugetlbfs -o "pagesize=1G,size=8G,uid=$(id -u),gid=$(id -g)" none /mnt/huge1g
    grep -H . /sys/kernel/mm/hugepages/hugepages-*/nr_hugepages >"$res/hugepages.txt"
    local tuned=(AERON_TERM_BUFFER_SPARSE_FILE=false AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true)
    for rep in $(seq "${BENCH3_REPS:-5}"); do
        for mode in $(rotate "$rep" off 2m 1g); do
            local extra=()
            run_base=
            case $mode in
                2m) run_base=/mnt/huge2m extra=(AERON_FILE_PAGE_SIZE=2097152) ;;
                1g) run_base=/mnt/huge1g extra=(AERON_FILE_PAGE_SIZE=1073741824) ;;
            esac
            for t in ipc tput; do run "huge-$mode" "h-$t" impr "$t" "${tuned[@]}" ${extra[@]+"${extra[@]}"}; done
        done
    done
    run_base=
    log "bench-1g done"
}

# k3s on this VM, then three pods run the harness with AERON_DIR on a Memory (tmpfs), a
# HugePages-2Mi and a HugePages-1Gi emptyDir. The huge pages bench-1g reserved are already
# there when k3s starts, which kubelet needs to report them.
k8s() {
    local p phase
    for p in /mnt/huge2m /mnt/huge1g; do if mountpoint -q "$p"; then sudo umount "$p"; fi; done
    # kubelet reports only the huge pages reserved before it starts
    sudo sysctl -q -w vm.nr_hugepages=1024
    echo 8 | sudo tee /sys/kernel/mm/hugepages/hugepages-1048576kB/nr_hugepages >/dev/null
    grep -H . /sys/kernel/mm/hugepages/hugepages-*/nr_hugepages >"$res/k8s-hugepages.txt"
    k3s_up
    for p in memory hugepages-2mi hugepages-1gi; do
        log "pod $p"
        if ! kubectl apply -f "$lab/harness/k8s/$p.yaml" >"$res/k8s-$p.apply" 2>&1; then
            log "pod $p not created: $(cat "$res/k8s-$p.apply")"
            continue
        fi
        for _ in $(seq 450); do
            phase=$(kubectl get pod "$p" -o jsonpath='{.status.phase}' 2>/dev/null)
            if [[ $phase == Succeeded || $phase == Failed ]]; then break; fi
            sleep 2
        done
        kubectl logs "$p" >"$res/k8s-$p.log" 2>&1
        kubectl describe pod "$p" >"$res/k8s-$p.describe" 2>&1
        grep -E '^(rtt|tput),' "$res/k8s-$p.log" | sed "s/^/$host,k8s-$p,hot23,0,/" | tee -a "$res/bench.csv" || true
        kubectl delete pod "$p" --wait=true >/dev/null 2>&1 || true
    done
    log "k8s done"
}

# On the pong host of a cross-host pair: a driver on CPU 0 and pong pinned to CPU 1, left
# running after this returns. pong_up <ping endpoint> <pong endpoint> [env...]
pong_up() {
    local ping_ep=$1 pong_ep=$2 dir
    shift 2
    dir=$shm/x86lab-pong-$(date +%s%N)
    setsid env AERON_DIR="$dir" AERON_DIR_DELETE_ON_START=true AERON_DIR_DELETE_ON_SHUTDOWN=true "$@" \
        taskset -c 0 "$bin/media_driver" </dev/null >"$res/pong-driver.log" 2>&1 &
    echo $! >"$res/pong-driver.pid"
    for _ in $(seq 100); do
        if [[ -e $dir/cnc.dat ]]; then break; fi
        sleep 0.1
    done
    sleep 0.5
    setsid env "$@" AERON_DIR="$dir" taskset -c 0 "$bin/impr/rtt" xpong "$ping_ep" "$pong_ep" 1 \
        </dev/null >"$res/pong.log" 2>&1 &
    echo $! >"$res/pong.pid"
}

pong_down() {
    local driver_pid
    driver_pid=$(cat "$res/pong-driver.pid")
    kill "$(cat "$res/pong.pid")" 2>/dev/null || true
    kill -INT "$driver_pid" 2>/dev/null || true
    for _ in $(seq 100); do
        if ! kill -0 "$driver_pid" 2>/dev/null; then break; fi
        sleep 0.1
    done
}

# busy_poll <µs>: kernel socket busy polling on both hosts (0 turns it off)
busy_poll() {
    sudo sysctl -q -w net.core.busy_read="$1" net.core.busy_poll="$1"
    "${peer_ssh[@]}" "sudo sysctl -q -w net.core.busy_read=$1 net.core.busy_poll=$1"
}

# UDP ping-pong between two hosts: ping here, pong on LAB_PEER_IP (the same region, network
# and placement group). Each host has 2 vCPUs: the driver on CPU 0, the hot thread on CPU 1.
bench_xhost() {
    local peer=${LAB_PEER_IP:?LAB_PEER_IP: the pong host} self line k
    self=$(hostname -I | awk '{print $1}')
    ping=1 pong=- hk=0 layout=xhost
    peer_ssh=(ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new "$peer")
    local ping_ep=$peer:20123 pong_ep=$self:20124
    local tuned=(AERON_TERM_BUFFER_SPARSE_FILE=false AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true)
    {
        lscpu | grep -E 'Model name|^CPU\(s\)|Thread'
        ip -br link
        echo "self=$self peer=$peer"
    } >"$res/xhost-system.txt" 2>&1
    for rep in $(seq "${BENCH3_REPS:-5}"); do
        for k in $(rotate "$rep" shared-noop shared-backoff shared-noop-busypoll); do
            local envs=()
            case $k in
                shared-noop | shared-noop-busypoll) envs=(AERON_THREADING_MODE=SHARED AERON_SHARED_IDLE_STRATEGY=noop "${tuned[@]}") ;;
                shared-backoff) envs=(AERON_THREADING_MODE=SHARED "${tuned[@]}") ;;
            esac
            if [[ $k == *busypoll ]]; then busy_poll 50; fi
            "${peer_ssh[@]}" "/srv/x86lab/harness/vm.sh pong-up $ping_ep $pong_ep ${envs[*]}"
            driver_start "${envs[@]}"
            line=$(env "${envs[@]}" AERON_DIR="$run_dir" LABEL="$k" taskset -c "$hk" timeout 300 \
                "$bin/impr/rtt" xping "$ping_ep" "$pong_ep" "$UDP_N" "$UDP_W" "$ping" 2>>"$res/client-errors.log") ||
                line="error,$k,xping"
            driver_stop
            "${peer_ssh[@]}" "/srv/x86lab/harness/vm.sh pong-down"
            if [[ $k == *busypoll ]]; then busy_poll 0; fi
            echo "$host,xhost,$layout,$rep,$line" | tee -a "$res/bench.csv"
        done
    done
    log "bench-xhost done"
}

# wait until the node is Ready and takes pods (it refuses them until the controller manager
# has made the default service account)
k3s_ready() {
    export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
    for _ in $(seq 90); do
        if kubectl get node 2>/dev/null | grep -q ' Ready'; then break; fi
        sleep 2
    done
    for _ in $(seq 90); do
        if kubectl get serviceaccount default >/dev/null 2>&1; then break; fi
        sleep 2
    done
}

# k3s on this VM, and what the pods mount from /srv/x86lab/k8s: the host's harness, driver
# and pod scripts, with the two libraries the static builds still load
k3s_up() {
    curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC="--disable traefik --disable metrics-server --write-kubeconfig-mode 644" \
        sh - >"$res/k3s-install.log" 2>&1
    k3s_ready
    kubectl get node -o jsonpath='{.items[0].status.allocatable}' >"$res/k8s-allocatable.json" 2>&1
    mkdir -p "$lab/k8s/lib"
    cp "$bin/impr/rtt" "$bin/impr/tput" "$bin/media_driver" "$lab/harness/k8s-run.sh" "$lab/k8s/"
    mkdir -p "$lab/k8s/k8s-cpu"
    cp "$lab/harness/k8s-cpu/"* "$lab/k8s/k8s-cpu/"
    { k3s --version; kubectl version; } >"$res/k8s-version.txt" 2>&1
    cp -L /usr/lib/x86_64-linux-gnu/libbsd.so.0 /usr/lib/x86_64-linux-gnu/libmd.so.0 "$lab/k8s/lib/"
    ldd "$lab/k8s/rtt" >"$res/k8s-ldd.txt" 2>&1
}

# cpu_pod <name>: the two-container pod of k8s-cpu/ until it ends; rows to bench.csv and each
# container's CPU throttling counters to throttle.csv
cpu_pod() {
    local name=$1 phase
    log "pod $name"
    sed "s/^  name: NAME$/  name: $name/" "$lab/harness/k8s-cpu/pod.yaml" >"$res/k8s-$name.yaml"
    if ! kubectl apply -f "$res/k8s-$name.yaml" >"$res/k8s-$name.apply" 2>&1; then
        log "pod $name not created: $(cat "$res/k8s-$name.apply")"
        return
    fi
    for _ in $(seq 450); do
        phase=$(kubectl get pod "$name" -o jsonpath='{.status.phase}' 2>/dev/null)
        if [[ $phase == Succeeded || $phase == Failed ]]; then break; fi
        sleep 2
    done
    kubectl logs "$name" -c app >"$res/k8s-$name-app.log" 2>&1
    kubectl logs "$name" -c driver >"$res/k8s-$name-driver.log" 2>&1
    kubectl describe pod "$name" >"$res/k8s-$name.describe" 2>&1
    sudo cat /var/lib/kubelet/cpu_manager_state >"$res/k8s-$name-cpu-manager-state.json" 2>&1 || true
    grep -E '^(rtt|tput),' "$res/k8s-$name-app.log" | sed "s/^/$host,k8s-$name,pod,0,/" | tee -a "$res/bench.csv" || true
    grep -h '^throttle,' "$res/k8s-$name-app.log" "$res/k8s-$name-driver.log" | sed "s/^/$host,$name,/" >>"$res/throttle.csv" || true
    kubectl delete pod "$name" --wait=true >/dev/null 2>&1 || true
}

# A host baseline with a SHARED noop driver, then the k8s-cpu/ pod (driver on one whole CPU,
# app on two) under kubelet's default CPU manager, then under the static policy with CPU 0
# reserved for the system
k8s_cpu() {
    state=pinned
    ping=2 pong=3 hk=0,1 layout=hot23
    # kubelet reports only the huge pages reserved before it starts
    sudo sysctl -q -w vm.nr_hugepages=1024
    local shared=(AERON_THREADING_MODE=SHARED AERON_SHARED_IDLE_STRATEGY=noop AERON_TERM_BUFFER_SPARSE_FILE=false
        AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true AERON_FILE_PAGE_SIZE=2097152)
    host_shared host-shared "${shared[@]}"
    k3s_up
    cpu_pod cpu-default
    log "switch kubelet to the static CPU manager"
    printf '%s\n' 'kubelet-arg:' '  - cpu-manager-policy=static' '  - reserved-cpus=0' |
        sudo tee /etc/rancher/k3s/config.yaml >/dev/null
    sudo systemctl stop k3s
    # kubelet will not start with a CPU manager state left by another policy
    sudo rm -f /var/lib/kubelet/cpu_manager_state
    sudo systemctl start k3s
    k3s_ready
    # their CPU requests would leave less than the pod's three whole CPUs; the pod needs neither
    kubectl -n kube-system scale deployment coredns local-path-provisioner --replicas=0 >>"$res/k8s-scale.log" 2>&1 || true
    kubectl -n kube-system wait --for=delete pod --all --timeout=120s >>"$res/k8s-scale.log" 2>&1 || true
    kubectl describe node >"$res/k8s-node-static.txt" 2>&1
    cpu_pod cpu-static
    # the same baseline with k3s's own processes running beside it
    host_shared host-shared-k3s "${shared[@]}"
    log "k8s-cpu done"
}

# host_shared <group> [env...]: 3 reps of ipc, udp and tput on the host with AERON_DIR on 2 MiB
# hugetlbfs, unmounted afterwards so kubelet can hand the pages to pods
host_shared() {
    local group=$1
    shift
    log "host baseline $group"
    sudo mkdir -p /mnt/huge2m
    mountpoint -q /mnt/huge2m || sudo mount -t hugetlbfs -o "pagesize=2M,size=2G,uid=$(id -u),gid=$(id -g)" none /mnt/huge2m
    run_base=/mnt/huge2m
    for rep in 1 2 3; do
        for t in ipc udp tput; do run "$group" "h-$t" impr "$t" "$@"; done
    done
    run_base=
    sudo umount /mnt/huge2m
}

test_phase() {
    cd "$lab/rusteron"
    # release C (-O3 -march=native) as users ship, without a fat-LTO link per test binary
    export CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 CARGO_TARGET_DIR=$lab/target-test
    local rc
    log "workspace tests, release"
    rc=0; timeout 3000 cargo test --workspace --release --no-fail-fast -- --nocapture >"$res/test-workspace-release.log" 2>&1 || rc=$?
    echo "workspace-release $rc" | tee -a "$res/tests.txt"
    if [[ ${LAB_TESTS:-} == workspace ]]; then
        log "test done (workspace only)"
        return
    fi

    log "slow archive tests (ignored by default)"
    rc=0; timeout 1800 cargo test --release -p rusteron-archive --lib --no-fail-fast -- --ignored --nocapture >"$res/test-slow.log" 2>&1 || rc=$?
    echo "slow $rc" | tee -a "$res/tests.txt"

    log "examples"
    for e in archive_error_handling duty_cycle persistent_subscription persistent_subscription_failover \
        record_and_replay recording_replication recording_throughput replay_merge; do
        rc=0; timeout 600 cargo run --release -p rusteron-archive --example "$e" >"$res/example-$e.log" 2>&1 || rc=$?
        echo "example $e $rc" | tee -a "$res/tests.txt"
    done
    for e in basic_pub_sub zero_copy_claim retained_images request_response file_transfer; do
        rc=0; timeout 600 cargo run --release -p rusteron-client --example "$e" >"$res/example-$e.log" 2>&1 || rc=$?
        echo "example $e $rc" | tee -a "$res/tests.txt"
    done
    rc=0; timeout 600 cargo run --release -p rusteron-client --features examples --example embedded_ping_pong >"$res/example-embedded_ping_pong.log" 2>&1 || rc=$?
    echo "example embedded_ping_pong $rc" | tee -a "$res/tests.txt"

    log "archive tests under AddressSanitizer (CI runs the client's)"
    local asan
    asan=$(find "$(clang -print-resource-dir)" -name 'libclang_rt.asan-x86_64.so' -o -name 'libclang_rt.asan.so' | head -1)
    # the runtime is found by path rather than preloaded, so the Java archive does not load it
    rc=0; env CC=clang CXX=clang++ RUSTERON_SANITIZE=address RUSTFLAGS="-C target-cpu=x86-64" \
        ASAN_OPTIONS=detect_leaks=0:verify_asan_link_order=0:allocator_may_return_null=1 \
        LD_LIBRARY_PATH="$(dirname "$asan")" CARGO_TARGET_DIR=$lab/target-asan \
        timeout 3000 cargo test -p rusteron-archive --features sanitize-address --lib -- --nocapture \
        >"$res/test-asan-archive.log" 2>&1 || rc=$?
    echo "asan-archive $rc" | tee -a "$res/tests.txt"
    log "test done"
}

case ${1:-} in
    bootstrap) bootstrap ;;
    build) build ;;
    bench) bench ;;
    abudp) abudp ;;
    bench-pinned) bench3 pinned ;;
    isolate) isolate ;;
    bench-isolated) bench3 isolated ;;
    bench-huge) BENCH3_SAMPLES=0 bench3 pinned ;;
    bench-1g) bench_1g ;;
    k8s) k8s ;;
    k8s-cpu) k8s_cpu ;;
    pong-up) shift; pong_up "$@" ;;
    pong-down) pong_down ;;
    bench-xhost) bench_xhost ;;
    test) test_phase ;;
    *) echo "usage: $0 bootstrap|build|bench|abudp|bench-pinned|isolate|bench-isolated|test" >&2; exit 2 ;;
esac
