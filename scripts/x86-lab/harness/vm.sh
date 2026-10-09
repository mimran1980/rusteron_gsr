#!/usr/bin/env bash
# Runs on a lab VM: vm.sh bootstrap|build|bench|test. Everything lives in /srv/x86lab;
# results go to /srv/x86lab/results, which the Mac copies after each phase.
set -euo pipefail

lab=${X86LAB:-/srv/x86lab}
shm=${SHM:-/dev/shm}
res=$lab/results
bin=$lab/bin
mkdir -p "$res" "$bin"
# /usr/sbin: Debian keeps ethtool and sysctl there, off a normal user's PATH
export RUSTUP_TOOLCHAIN=1.95.0 CARGO_TERM_COLOR=never PATH=$HOME/.cargo/bin:$PATH:/usr/sbin:/sbin
host=$(hostname)
native="-C target-cpu=native"

log() { echo "[$(date +%T)] $*"; }

bootstrap() {
    sudo apt-get update -qq
    sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq build-essential cmake clang libclang-dev \
        pkg-config libbsd-dev uuid-dev zlib1g-dev libssl-dev default-jdk-headless curl util-linux ethtool \
        fio sysstat xfsprogs nftables >/dev/null
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
    for b in rtt tput pspoll rec arcload; do
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
    # the builds leave a Gradle daemon behind, a JVM that would sit beside every benchmark
    pkill -f org.gradle.launcher.daemon || true
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
    sed "s/^  name: NAME$/  name: $1/" "$lab/harness/k8s-cpu/pod.yaml" >"$res/k8s-$1.yaml"
    run_pod "$1"
}

# run_pod <name>: applies $res/k8s-<name>.yaml and collects as cpu_pod describes
run_pod() {
    local name=$1 phase
    log "pod $name"
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

# --- 8 vCPUs ------------------------------------------------------------------------------

# Core layout of the online CPUs. hk8: CPU 0 alone, for housekeeping; every other CPU can be
# isolated. Ping, pong and the driver's sender each get a core of their own; the receiver too
# where there are enough cores, else CPU 0's SMT sibling (Intel's 4 cores), which shares its
# core with the housekeeping CPU rather than with a spinning sender, else none (SMT off).
topo8() {
    eval "$(python3 - <<'PY'
import glob
def expand(s):
    out = []
    for part in s.strip().split(','):
        a, _, b = part.partition('-')
        out += range(int(a), int(b or a) + 1)
    return out
online = set(expand(open('/sys/devices/system/cpu/online').read()))
cores = sorted({tuple(c for c in expand(open(f'/sys/devices/system/cpu/cpu{n}/topology/thread_siblings_list').read()) if c in online) for n in online})
core0 = next(c for c in cores if 0 in c)
hot = [c[0] for c in cores if 0 not in c]
# with SMT off on 4 cores there is no CPU left for a receiver of its own
rcv = hot[3] if len(hot) > 3 else (core0[1] if len(core0) > 1 else '')
cpus = sorted(online)
j = lambda l: ','.join(map(str, l))
# across hosts each host has one app thread, so app, sender and receiver take the first 3 cores
xrcv = hot[2] if len(hot) > 2 else (core0[1] if len(core0) > 1 else '')
print(f"hk8=0 first_hk8=0 ping8={hot[0]} pong8={hot[1]} snd8={hot[2]} rcv8={rcv} "
      f"xapp8={hot[0]} xsnd8={hot[1]} xrcv8={xrcv} "
      f"all8={j(cpus)} iso8={j([c for c in cpus if c != 0])} smt8={int(any(len(c) > 1 for c in cores))}")
PY
)"
    echo "hk=$hk8 ping=$ping8 pong=$pong8 sender=$snd8 receiver=$rcv8 isolatable=$iso8 smt=$smt8" >"$res/layouts8.txt"
    echo "across hosts: app=$xapp8 sender=$xsnd8 receiver=$xrcv8" >>"$res/layouts8.txt"
    sort -u /sys/devices/system/cpu/cpu*/topology/thread_siblings_list >>"$res/layouts8.txt"
}

# AERON_DIR on 2 MiB hugetlbfs for the 8-vCPU host runs
huge8() {
    sudo sysctl -q -w vm.nr_hugepages=1024
    sudo mkdir -p /mnt/huge2m
    mountpoint -q /mnt/huge2m || sudo mount -t hugetlbfs -o "pagesize=2M,size=2G,uid=$(id -u),gid=$(id -g)" none /mnt/huge2m
    run_base=/mnt/huge2m
}

# run8 <group> <label> <ipc|udp|tput> <driver cpus> <client cpus> <ping> <pong> [env...]: as
# run, with the driver and client processes on their own CPU lists; "-" leaves one unpinned
run8() {
    local group=$1 label=$2 test=$3 dmask=$4 cmask=$5 p=$6 q=$7 cmd line
    shift 7
    case $test in
        ipc) cmd=(rtt ipc "$IPC_N" "$IPC_W" "$p" "$q") ;;
        udp) cmd=(rtt udp "$UDP_N" "$UDP_W" "$p" "$q") ;;
        tput) cmd=(tput 5 "$p" "$q") ;;
    esac
    [[ $dmask == - ]] && dmask=$all8
    [[ $cmask == - ]] && cmask=$all8
    hk=$dmask driver_start "$@"
    line=$(env "$@" AERON_DIR="$run_dir" LABEL="$label" taskset -c "$cmask" timeout 300 "$bin/impr/${cmd[0]}" \
        "${cmd[@]:1}" 2>>"$res/client-errors.log") || line="error,$label,$test"
    driver_stop
    echo "$host,$group,$layout,$rep,$line" | tee -a "$res/bench.csv"
}

# What pinning buys IPC on 8 vCPUs (UDP is measured between two hosts, in xhost8), one change
# per variant: none: nothing pinned. client: ping and pong pinned, the client's other threads
# on CPU 0. conductor-hot: those other threads (its conductor) on the ping and pong CPUs
# instead. shared-1cpu: a SHARED noop driver on a CPU of its own. The driver otherwise keeps
# its default idle strategies, so no unpinned thread spins. bench8 <state>
bench8() {
    state=$1 layout=$1
    topo8
    huge8
    local base=(AERON_TERM_BUFFER_SPARSE_FILE=false AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true AERON_FILE_PAGE_SIZE=2097152)
    local shared=("${base[@]}" AERON_THREADING_MODE=SHARED AERON_SHARED_IDLE_STRATEGY=noop)
    local variants=(none client conductor-hot shared-1cpu)
    if [[ $state != pinned ]]; then
        # only variants that pin every busy thread: isolated CPUs take no unpinned work, which
        # would all crowd onto CPU 0
        variants=(client conductor-hot shared-1cpu)
        if [[ $state == tuned* ]]; then tune8_runtime; fi
    fi
    {
        echo "state=$state kernel=$(uname -r)"
        cat /proc/cmdline
        echo "isolated=$(cat /sys/devices/system/cpu/isolated) nohz_full=$(cat /sys/devices/system/cpu/nohz_full 2>/dev/null)"
        grep -H . /sys/devices/system/cpu/vulnerabilities/* 2>/dev/null
        echo "thp=$(cat /sys/kernel/mm/transparent_hugepage/enabled) watchdog=$(sysctl -n kernel.watchdog) workqueue=$(cat /sys/devices/virtual/workqueue/cpumask 2>/dev/null)"
        echo "online=$(cat /sys/devices/system/cpu/online) smt=$(cat /sys/devices/system/cpu/smt/control 2>/dev/null) idle=$(cat /sys/devices/system/cpu/cpuidle/current_driver 2>/dev/null)"
        echo "clocksource=$(cat /sys/devices/system/clocksource/clocksource0/current_clocksource) ksm=$(cat /sys/kernel/mm/ksm/run 2>/dev/null) printk=$(sysctl -n kernel.printk)"
        grep -E 'CONFIG_INIT_ON_ALLOC_DEFAULT_ON|CONFIG_INIT_ON_FREE_DEFAULT_ON' "/boot/config-$(uname -r)" 2>/dev/null
        echo "scheduled events: $(curl -s -m 5 -H Metadata:true --noproxy '*' 'http://169.254.169.254/metadata/scheduledevents?api-version=2020-07-01')"
        cat "$res/layouts8.txt"
    } >"$res/state8-$state.txt" 2>&1
    cat /proc/interrupts >"$res/interrupts8-$state-before.txt"
    for rep in $(seq "${BENCH8_REPS:-5}"); do
        for v in $(rotate "$rep" "${variants[@]}"); do
            for t in ipc tput; do
                case $v in
                    none) run8 "$state-$v" "h-$t" "$t" - - - - "${base[@]}" ;;
                    client) run8 "$state-$v" "h-$t" "$t" "$hk8" "$hk8" "$ping8" "$pong8" "${base[@]}" ;;
                    conductor-hot) run8 "$state-$v" "h-$t" "$t" "$hk8" "$ping8,$pong8" "$ping8" "$pong8" "${base[@]}" ;;
                    shared-1cpu) run8 "$state-$v" "h-$t" "$t" "$snd8" "$hk8" "$ping8" "$pong8" "${shared[@]}" ;;
                esac
            done
        done
    done
    cat /proc/interrupts >"$res/interrupts8-$state-after.txt"
    run_base=
    sudo umount /mnt/huge2m
    log "bench8 $state done"
}

# xhost_env <variant>: the driver's environment (xenv) and CPUs (xmask) for a cross-host variant.
# ded-threads: noop sender and receiver each pinned to a core, the conductor on CPU 0.
# sharednet-threads: one noop network thread pinned, the conductor on CPU 0. shared-1cpu: a
# SHARED noop driver on one CPU. ded-threads-busyread: ded-threads with socket busy polling.
# Each uses the socket profile of Adaptive's low-latency driver (2 MiB socket buffers and
# initial receiver window) unless it ends in -defaults, which keeps Aeron's (128 KiB receive
# buffer and window, the OS's send buffer). ded-threads-irqrcv: the NIC's IRQs on the receiver's
# core. xtput-jumbo: MTU 9000 and Aeron MTU 8192; xtput-iov16: 16-message io vectors and sends.
# ded-threads-wide: 16 MiB socket buffers and initial receiver window over 64 MiB terms, which a
# long round trip needs to keep the link full. AERON_DIR is on 2 MiB hugetlbfs throughout.
xhost_env() {
    local base=(AERON_TERM_BUFFER_SPARSE_FILE=false AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true AERON_FILE_PAGE_SIZE=2097152)
    if [[ $1 != *-defaults ]]; then
        base+=(AERON_SOCKET_SO_SNDBUF=2097152 AERON_SOCKET_SO_RCVBUF=2097152 AERON_RCV_INITIAL_WINDOW_LENGTH=2097152)
    fi
    if [[ $1 == *-wide ]]; then
        base+=(AERON_SOCKET_SO_SNDBUF=16777216 AERON_SOCKET_SO_RCVBUF=16777216 AERON_RCV_INITIAL_WINDOW_LENGTH=16777216
            AERON_TERM_BUFFER_LENGTH=67108864)
    fi
    case $1 in
        xtput-jumbo) base+=(AERON_MTU_LENGTH=8192) ;;
        xtput-iov16)
            base+=(AERON_SENDER_IO_VECTOR_CAPACITY=16 AERON_RECEIVER_IO_VECTOR_CAPACITY=16
                AERON_NETWORK_PUBLICATION_MAX_MESSAGES_PER_SEND=16) ;;
    esac
    case ${1%-defaults} in
        ded-threads | ded-threads-wide | ded-threads-busyread | ded-threads-irqrcv | ded-threads-busyread-irqrcv | xtput | xtput-jumbo | xtput-iov16)
            xenv=("${base[@]}" AERON_SENDER_IDLE_STRATEGY=noop AERON_RECEIVER_IDLE_STRATEGY=noop
                AERON_CONDUCTOR_CPU_AFFINITY="$first_hk8" AERON_SENDER_CPU_AFFINITY="$xsnd8" AERON_RECEIVER_CPU_AFFINITY="$xrcv8")
            xmask=$hk8 ;;
        sharednet-threads | xtput-sharednet)
            xenv=("${base[@]}" AERON_THREADING_MODE=SHARED_NETWORK AERON_SHAREDNETWORK_IDLE_STRATEGY=noop
                AERON_CONDUCTOR_CPU_AFFINITY="$first_hk8" AERON_SENDER_CPU_AFFINITY="$xsnd8")
            xmask=$hk8 ;;
        shared-1cpu | xtput-shared)
            xenv=("${base[@]}" AERON_THREADING_MODE=SHARED AERON_SHARED_IDLE_STRATEGY=noop)
            xmask=$xsnd8 ;;
    esac
}

# On the pong host of a cross-host pair: the driver laid out as on the ping host and pong pinned
# to the app CPU, left running after this returns. pong_up8 <variant> <ping endpoint> <pong endpoint>
pong_up8() {
    local v=$1 ping_ep=$2 pong_ep=$3 dir
    topo8
    huge8
    xhost_env "$v"
    dir=$run_base/x86lab-pong-$(date +%s%N)
    setsid env AERON_DIR="$dir" AERON_DIR_DELETE_ON_START=true AERON_DIR_DELETE_ON_SHUTDOWN=true "${xenv[@]}" \
        taskset -c "$xmask" "$bin/media_driver" </dev/null >"$res/pong-driver.log" 2>&1 &
    echo $! >"$res/pong-driver.pid"
    for _ in $(seq 100); do
        if [[ -e $dir/cnc.dat ]]; then break; fi
        sleep 0.1
    done
    sleep 0.5
    setsid env "${xenv[@]}" AERON_DIR="$dir" taskset -c "$hk8" "$bin/impr/rtt" xpong "$ping_ep" "$pong_ep" "$xapp8" \
        </dev/null >"$res/pong.log" 2>&1 &
    echo $! >"$res/pong.pid"
}

# On the subscriber host of a cross-host pair: the driver for <variant> and a throughput
# subscriber bound to <endpoint> for 5 one-second samples, its line in xsub.out.
# xsub_up8 <variant> <endpoint>
xsub_up8() {
    local v=$1 ep=$2 dir
    topo8
    huge8
    xhost_env "$v"
    dir=$run_base/x86lab-xsub-$(date +%s%N)
    setsid env AERON_DIR="$dir" AERON_DIR_DELETE_ON_START=true AERON_DIR_DELETE_ON_SHUTDOWN=true "${xenv[@]}" \
        taskset -c "$xmask" "$bin/media_driver" </dev/null >"$res/pong-driver.log" 2>&1 &
    echo $! >"$res/pong-driver.pid"
    for _ in $(seq 100); do
        if [[ -e $dir/cnc.dat ]]; then break; fi
        sleep 0.1
    done
    sleep 0.5
    rm -f "$res/xsub.out"
    setsid env "${xenv[@]}" AERON_DIR="$dir" LABEL="$v" taskset -c "$hk8" timeout 60 "$bin/impr/tput" xsub "$ep" 5 "$xapp8" \
        </dev/null >"$res/xsub.out" 2>>"$res/client-errors.log" &
    echo $! >"$res/pong.pid"
}

# the subscriber's line once it has finished, waiting up to 30 s
xsub_result() {
    local pid
    pid=$(cat "$res/pong.pid")
    for _ in $(seq 60); do
        if ! kill -0 "$pid" 2>/dev/null; then break; fi
        sleep 0.5
    done
    cat "$res/xsub.out"
}

# xhost_prepare <variant> <on|off>: before (on) and after (off) one cross-host run, on both hosts:
# IRQs re-pinned in the isolated and tuned states, the variant's network settings, and the
# VF, busy-poll and datapath counters, whose change over the run goes to xhost8-runs.csv
xhost_prepare() {
    local v=$1 when=$2 settings=() setting
    if [[ $v == *busyread* ]]; then settings+=(busy-net); fi
    if [[ $v == *irqrcv* ]]; then settings+=(irq-rcv); fi
    if [[ $v == *jumbo* ]]; then settings+=(jumbo); fi
    if [[ $when == on ]]; then
        if [[ $state != pinned ]]; then
            pin_irqs "$hk8"
            "${peer_ssh[@]}" "/srv/x86lab/harness/vm.sh pin-irqs"
        fi
        for setting in ${settings[@]+"${settings[@]}"}; do
            "$lab/harness/vm.sh" "$setting" on
            "${peer_ssh[@]}" "/srv/x86lab/harness/vm.sh $setting on"
        done
        stats_before="$(path_stats) $("${peer_ssh[@]}" /srv/x86lab/harness/vm.sh path-stats)"
    else
        local after
        after="$(path_stats) $("${peer_ssh[@]}" /srv/x86lab/harness/vm.sh path-stats)"
        awk -v h="$host,$state,$v,$rep" -v a="$after" -v b="$stats_before" \
            'BEGIN { n = split(a, x, " "); split(b, y, " "); printf "%s", h; for (i = 1; i <= n; i++) printf ",%d", x[i] - y[i]; print "" }' \
            >>"$res/xhost8-runs.csv"
        for setting in ${settings[@]+"${settings[@]}"}; do
            "$lab/harness/vm.sh" "$setting" off
            "${peer_ssh[@]}" "/srv/x86lab/harness/vm.sh $setting off"
        done
    fi
}

# UDP between the two hosts of a pair, each laid out by topo8: round trips (ping here, pong on
# LAB_PEER_IP) for each driver variant, then throughput (publisher here, subscriber there) with
# Adaptive's socket profile and with Aeron's defaults. Both hosts are in the state named, which
# lab.sh brought them to. xhost8 <state>
xhost8() {
    state=$1 layout=$1
    local peer=${LAB_PEER_IP:?LAB_PEER_IP: the pong host} self line v
    topo8
    self=$(hostname -I | awk '{print $1}')
    peer_ssh=(ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new "$peer")
    local ping_ep=$peer:20123 pong_ep=$self:20124
    {
        echo "self=$self peer=$peer kernel=$(uname -r)"
        cat /proc/cmdline
        cat "$res/layouts8.txt"
        echo "clocksource=$(cat /sys/devices/system/clocksource/clocksource0/current_clocksource) of $(cat /sys/devices/system/clocksource/clocksource0/available_clocksource)"
        ip -br link
        for i in $(ls /sys/class/net | grep -v '^lo$'); do
            echo "== $i"
            ethtool -i "$i" 2>&1 | head -3
            ethtool -c "$i" 2>&1 | grep -E 'Adaptive|rx-usecs|tx-usecs' || true
            ethtool -l "$i" 2>&1 | tail -5 || true
            ethtool -k "$i" 2>&1 | grep -E '^(generic-receive-offload|large-receive-offload|tcp-segmentation-offload):' || true
        done
        sysctl net.core.busy_read net.core.busy_poll
    } >"$res/xhost8-nic-$state.txt" 2>&1
    local variants=(ded-threads ded-threads-defaults sharednet-threads shared-1cpu ded-threads-busyread ded-threads-irqrcv
        ded-threads-busyread-irqrcv)
    huge8
    echo "host,state,variant,rep,vf_rx,vf_tx,busy_poll_rx,path_switches,peer_vf_rx,peer_vf_tx,peer_busy_poll_rx,peer_path_switches" \
        >>"$res/xhost8-runs.csv"
    for rep in $(seq "${BENCH8_REPS:-5}"); do
        for v in $(rotate "$rep" xtput xtput-defaults xtput-jumbo xtput-iov16 xtput-sharednet xtput-shared); do
            xhost_prepare "$v" on
            "${peer_ssh[@]}" "/srv/x86lab/harness/vm.sh xsub-up8 $v $ping_ep"
            xhost_env "$v"
            hk=$xmask driver_start "${xenv[@]}"
            env "${xenv[@]}" AERON_DIR="$run_dir" taskset -c "$hk8" timeout 60 "$bin/impr/tput" xpub "$ping_ep" 8 "$xapp8" \
                2>>"$res/client-errors.log" || log "xpub $v failed"
            driver_stop
            line=$("${peer_ssh[@]}" "/srv/x86lab/harness/vm.sh xsub-result") || true
            [[ -n $line ]] || line="error,$v,udp"
            "${peer_ssh[@]}" "/srv/x86lab/harness/vm.sh pong-down"
            xhost_prepare "$v" off
            echo "$host,xhost-$state-$v,$layout,$rep,$line" | tee -a "$res/bench.csv"
        done
        for v in $(rotate "$rep" "${variants[@]}"); do
            xhost_prepare "$v" on
            "${peer_ssh[@]}" "/srv/x86lab/harness/vm.sh pong-up8 $v $ping_ep $pong_ep"
            xhost_env "$v"
            hk=$xmask driver_start "${xenv[@]}"
            line=$(env "${xenv[@]}" AERON_DIR="$run_dir" LABEL="$v" taskset -c "$hk8" timeout 120 \
                "$bin/impr/rtt" xping "$ping_ep" "$pong_ep" "$UDP_N" "$UDP_W" "$xapp8" 2>>"$res/client-errors.log") ||
                line="error,$v,xping"
            driver_stop
            "${peer_ssh[@]}" "/srv/x86lab/harness/vm.sh pong-down"
            xhost_prepare "$v" off
            echo "$host,xhost-$state-$v,$layout,$rep,$line" | tee -a "$res/bench.csv"
        done
    done
    run_base=
    log "xhost8 $state done"
}

# loss <basis points|off> [peer ip]: drops that share of the UDP packets arriving from the peer,
# in nftables' input hook, so neither end is told, as with loss on the wire; off removes it
loss() {
    sudo nft delete table inet lab 2>/dev/null || true
    if [[ $1 == off ]]; then return; fi
    sudo nft add table inet lab &&
        sudo nft add chain inet lab in '{ type filter hook input priority 0; }' &&
        sudo nft add rule inet lab in ip saddr "$2" meta l4proto udp numgen random mod 10000 lt "$1" counter drop
}

# the packets the loss rule has dropped since it was added
loss_count() {
    sudo nft list table inet lab 2>/dev/null | awk '/counter/ { for (i = 1; i < NF; i++) if ($i == "packets") print $(i + 1) }'
}

# aeron_stat <aeron dir>: the driver's NAKs sent and received, retransmits sent and loss gap fills
aeron_stat() {
    local jars=$lab/rusteron/rusteron-archive/aeron
    timeout 4 java --add-opens java.base/jdk.internal.misc=ALL-UNNAMED -Daeron.dir="$1" \
        -cp "$jars/aeron-samples/build/libs/aeron-samples-1.52.2.jar:$jars/aeron-all/build/libs/aeron-all-1.52.2.jar" \
        io.aeron.samples.AeronStat 2>/dev/null |
        awk -F' - ' '{ split($1, a, ":"); v = a[2]; gsub(/[ ,]/, "", v); c[$2] = v }
            END { printf "%d,%d,%d,%d", c["NAKs sent"], c["NAKs received"], c["Retransmits sent"], c["Loss gap fills"] }' ||
        true # timeout always ends AeronStat, which never exits by itself
}

# Loss and distance: UDP between this host and LAB_PEER_IP, in the same zone (xnet8-zone) or in
# another region (xnet8-region), with ded-threads, and across regions also ded-threads-wide. For
# each share of UDP packets dropped on arrival at both hosts (none, 0.1%, 1%): throughput
# (publisher here) and round trips, with this host's NAK and retransmit counters and both
# hosts' drop counts in xnet8-counters.csv. Across regions a round trip takes tens of ms, so
# round trips run 60 s instead of 20 and only for ded-threads. xnet8 <zone|region>
xnet8() {
    local link=$1 peer=${LAB_PEER_IP:?LAB_PEER_IP: the peer host} self v bp t line counters
    state=$link layout=$link
    topo8
    self=$(hostname -I | awk '{print $1}')
    peer_ssh=(ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new "$peer")
    local ping_ep=$peer:20123 pong_ep=$self:20124 variants=(ded-threads) seconds=20 reps=${XNET_REPS:-3}
    if [[ $link == region ]]; then variants+=(ded-threads-wide) seconds=60 reps=${XNET_REPS:-2}; fi
    ping -c 20 -q "$peer" >"$res/xnet8-$link-ping.txt" 2>&1 || true
    huge8
    echo "host,link,variant,loss_bp,test,rep,naks_sent,naks_received,retransmits_sent,loss_gap_fills,dropped_here,dropped_there" \
        >>"$res/xnet8-counters.csv"
    # xnet_loss <bp>: the loss rule on both hosts, afresh so its counters start at 0
    xnet_loss() {
        if (($1 == 0)); then loss off; peer loss off; else loss "$1" "$peer"; peer loss "$1" "$self"; fi
    }
    xnet_record() {
        echo "$host,xnet-$link-$v-loss$bp,$layout,$rep,$line" | tee -a "$res/bench.csv"
        echo "$host,$link,$v,$bp,$t,$rep,$counters,$(loss_count),$(peer loss-count)" >>"$res/xnet8-counters.csv"
    }
    for rep in $(seq "$reps"); do
        for bp in 0 10 100; do
            for v in "${variants[@]}"; do
                t=tput
                xnet_loss "$bp"
                peer xsub-up8 "$v" "$ping_ep"
                xhost_env "$v"
                hk=$xmask driver_start "${xenv[@]}"
                env "${xenv[@]}" AERON_DIR="$run_dir" taskset -c "$hk8" timeout 60 "$bin/impr/tput" xpub "$ping_ep" 8 "$xapp8" \
                    2>>"$res/client-errors.log" || log "xpub $v loss $bp failed"
                counters=$(aeron_stat "$run_dir")
                driver_stop
                line=$(peer xsub-result) || true
                [[ -n $line ]] || line="error,$v,udp"
                peer pong-down
                xnet_record
                if [[ $v != ded-threads ]]; then continue; fi
                t=rtt
                xnet_loss "$bp"
                peer pong-up8 "$v" "$ping_ep" "$pong_ep"
                hk=$xmask driver_start "${xenv[@]}"
                line=$(env "${xenv[@]}" RTT_SECONDS="$seconds" AERON_DIR="$run_dir" LABEL="$v" taskset -c "$hk8" \
                    timeout $((seconds * 2 + 60)) "$bin/impr/rtt" xping "$ping_ep" "$pong_ep" "$UDP_N" "$UDP_W" "$xapp8" \
                    2>>"$res/client-errors.log") || line="error,$v,xping"
                counters=$(aeron_stat "$run_dir")
                driver_stop
                peer pong-down
                xnet_record
            done
        done
    done
    xnet_loss 0
    run_base=
    log "xnet8 $link done"
}

# A Java ArchivingMediaDriver, with AERON_DIR and the archive on tmpfs so that disk speed
# does not hide the CPUs. archive_start <jvm cpus|-> [java option...]
archive_start() {
    local mask=$1 jar=$lab/rusteron/rusteron-archive/aeron/aeron-all/build/libs/aeron-all-1.52.2.jar
    shift
    [[ $mask == - ]] && mask=$all8
    runs=$((runs + 1))
    run_dir=$shm/x86lab-$runs archive_dir=$shm/x86lab-archive-$runs
    taskset -c "$mask" java -Xms1g -Xmx1g -XX:+AlwaysPreTouch -XX:+UseParallelGC -XX:-UsePerfData \
        -XX:+UnlockDiagnosticVMOptions -XX:GuaranteedSafepointInterval=300000 --add-opens java.base/jdk.internal.misc=ALL-UNNAMED \
        -Dagrona.disable.bounds.checks=true -Daeron.dir="$run_dir" \
        -Daeron.dir.delete.on.start=true -Daeron.dir.delete.on.shutdown=true -Daeron.term.buffer.sparse.file=false \
        -Daeron.pre.touch.mapped.memory=true -Daeron.archive.dir="$archive_dir" -Daeron.archive.threading.mode=DEDICATED \
        -Daeron.archive.control.channel="$ARCHIVE_CONTROL" "-Daeron.archive.replication.channel=aeron:udp?endpoint=localhost:0" \
        -Daeron.archive.recording.events.enabled=false "$@" -cp "$jar" io.aeron.archive.ArchivingMediaDriver \
        >"$res/archive-last.log" 2>&1 &
    archive_pid=$!
    for _ in $(seq 100); do
        if [[ -e $run_dir/cnc.dat ]]; then break; fi
        sleep 0.1
    done
    sleep 2
}

archive_stop() {
    # SIGTERM: a background job of a non-interactive shell starts with SIGINT ignored, and the JVM keeps that
    kill -TERM "$archive_pid" 2>/dev/null || true
    wait "$archive_pid" 2>/dev/null || true
    rm -rf "$archive_dir"
}

# archive_rtt <tag>: the IPC ping-pong against the archive's driver, with Aeron's counters and a
# thread dump of the archive saved if it is still going after 45 s; it stops at 90 s
archive_rtt() {
    local out=$res/archive-rtt.out pid watcher samples
    samples=$lab/rusteron/rusteron-archive/aeron/aeron-samples/build/libs/aeron-samples-1.52.2.jar
    env AERON_DIR="$run_dir" LABEL="a-ipc" taskset -c "$hk8" timeout 90 "$bin/impr/rtt" ipc "$IPC_N" "$IPC_W" "$ping8" "$pong8" \
        >"$out" 2>>"$res/client-errors.log" &
    pid=$!
    (
        sleep 45
        if kill -0 "$pid" 2>/dev/null; then
            {
                echo "stalled: $1"
                local cp="$samples:$lab/rusteron/rusteron-archive/aeron/aeron-all/build/libs/aeron-all-1.52.2.jar"
                timeout 5 java --add-opens java.base/jdk.internal.misc=ALL-UNNAMED -cp "$cp" -Daeron.dir="$run_dir" \
                    io.aeron.samples.AeronStat
                timeout 10 java --add-opens java.base/jdk.internal.misc=ALL-UNNAMED -cp "$cp" -Daeron.dir="$run_dir" \
                    io.aeron.samples.ErrorStat
                jcmd "$archive_pid" Thread.print
            } >"$res/archive-stall-$1.txt" 2>&1
        fi
    ) >/dev/null 2>&1 &    # else it holds the caller's $(...) open for the whole 45 s
    watcher=$!
    wait "$pid" || return 1
    kill "$watcher" 2>/dev/null || true
    cat "$out"
}

# What pinning the Java archive buys: recording throughput (rec) and an IPC ping-pong whose
# ping stream is recorded. jvm-unpinned: nothing pinned. jvm-hk: the JVM on hk.
# jvm-threads: also its archive-recorder on a core of its own. jvm-threads-noop: also a
# noop idle strategy for that recorder, which then spins.
archive8() {
    topo8
    layout=pinned
    export ARCHIVE_CONTROL='aeron:udp?endpoint=localhost:8010'
    for rep in $(seq "${BENCH8_REPS:-5}"); do
        for v in $(rotate "$rep" jvm-unpinned jvm-hk jvm-threads jvm-threads-noop); do
            local mask=$hk8 opts=() pins=()
            case $v in
                jvm-unpinned) mask=- ;;
                jvm-threads) pins=(archive-recorder="$snd8") ;;
                jvm-threads-noop) pins=(archive-recorder="$snd8") opts=(-Daeron.archive.recorder.idle.strategy=noop) ;;
            esac
            for t in rec ipc; do
                archive_start "$mask" ${opts[@]+"${opts[@]}"}
                if ((${#pins[@]})) && ! pin_java "$archive_pid" "${pins[@]}"; then log "archive8 $v: pinning failed"; fi
                local line
                if [[ $t == rec ]]; then
                    line=$(env AERON_DIR="$run_dir" LABEL="a-rec" taskset -c "$hk8" timeout 300 "$bin/impr-ps/rec" tput 1000000 256 "$ping8" \
                        2>>"$res/client-errors.log") || line="error,a-rec,rec"
                else
                    env AERON_DIR="$run_dir" taskset -c "$hk8" timeout 60 "$bin/impr-ps/rec" start 1002 2>>"$res/client-errors.log" ||
                        log "archive8 $v: rec start failed"
                    line=$(archive_rtt "$v-$rep") || line="error,a-ipc,ipc"
                fi
                archive_stop
                echo "$host,archive-$v,$layout,$rep,$line" | tee -a "$res/bench.csv"
            done
        done
    done
    log "archive8 done"
}

# pod8 <name> <shared|dedicated> <driver cpus> <app cpus> <guaranteed|burstable> <exclusive 0|1>:
# writes and runs a pod of k8s-cpu/'s two containers with these CPU requests
pod8() {
    local name=$1 mode=$2 dcpu=$3 acpu=$4 qos=$5 excl=$6 dlim alim
    dlim="cpu: \"$dcpu\", " alim="cpu: \"$acpu\", "
    [[ $qos == burstable ]] && dlim= alim=
    cat >"$res/k8s-$name.yaml" <<YAML
apiVersion: v1
kind: Pod
metadata:
  name: $name
spec:
  restartPolicy: Never
  containers:
    - name: driver
      image: debian:trixie-slim
      command: ["/lab/k8s-cpu/driver.sh"]
      env:
        - {name: DRIVER_MODE, value: $mode}
        - {name: REPS, value: "5"}
        - {name: AERON_TERM_BUFFER_SPARSE_FILE, value: "false"}
        - {name: AERON_FILE_PAGE_SIZE, value: "2097152"}
        - {name: AERON_PERFORM_STORAGE_CHECKS, value: "false"}
      resources:
        requests: {cpu: "$dcpu", memory: 1Gi, hugepages-2Mi: 1Gi}
        limits: {${dlim}memory: 1Gi, hugepages-2Mi: 1Gi}
      volumeMounts:
        - {name: aeron, mountPath: /aeron}
        - {name: lab, mountPath: /lab}
    - name: app
      image: debian:trixie-slim
      command: ["/lab/k8s-cpu/app.sh"]
      env:
        - {name: REPS, value: "5"}
        - {name: EXCLUSIVE, value: "$excl"}
        - {name: APP_PING, value: "$ping8"}
        - {name: APP_PONG, value: "$pong8"}
        - {name: APP_MASK, value: "$hk8"}
        - {name: AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY, value: "true"}
      resources:
        requests: {cpu: "$acpu", memory: 1Gi, hugepages-2Mi: 1Gi}
        limits: {${alim}memory: 1Gi, hugepages-2Mi: 1Gi}
      volumeMounts:
        - {name: aeron, mountPath: /aeron}
        - {name: lab, mountPath: /lab}
  volumes:
    - name: aeron
      emptyDir: {medium: HugePages-2Mi}
    - name: lab
      hostPath: {path: /srv/x86lab/k8s, type: Directory}
YAML
    run_pod "$name"
}

# Pods with different CPU requests, under kubelet's default CPU manager and then the static
# one with CPU 0 reserved for the system. Names: <policy>-<driver: s shared, d dedicated, and
# CPUs>-a<app CPUs>[-burst]; burstable pods have requests but no CPU limits.
k8s8() {
    topo8
    # kubelet reports only the huge pages reserved before it starts
    sudo sysctl -q -w vm.nr_hugepages=2048
    k3s_up
    pod8 def-s1-a2 shared 1 2 guaranteed 0
    pod8 def-s2-a3 shared 2 3 guaranteed 0
    pod8 def-s1-a2.5 shared 1 2500m guaranteed 0
    pod8 def-s1-a2-burst shared 1 2 burstable 0
    log "switch kubelet to the static CPU manager"
    # no full-pcpus-only: on SMT it refuses any request that is not a whole number of cores
    printf '%s\n' 'kubelet-arg:' '  - cpu-manager-policy=static' "  - reserved-cpus=$hk8" |
        sudo tee /etc/rancher/k3s/config.yaml >/dev/null
    k3s_restart
    sudo cat /etc/rancher/k3s/config.yaml /var/lib/kubelet/cpu_manager_state >"$res/k8s8-static.txt" 2>&1 || true
    # their CPU requests would leave too little room; the pods need neither
    kubectl -n kube-system scale deployment coredns local-path-provisioner --replicas=0 >>"$res/k8s-scale.log" 2>&1 || true
    kubectl -n kube-system wait --for=delete pod --all --timeout=120s >>"$res/k8s-scale.log" 2>&1 || true
    kubectl describe node >"$res/k8s8-node-static.txt" 2>&1
    pod8 st-s1-a2 shared 1 2 guaranteed 1
    pod8 st-s2-a3 shared 2 3 guaranteed 1
    pod8 st-d3-a3 dedicated 3 3 guaranteed 1
    pod8 st-s1-a2.5 shared 1 2500m guaranteed 0
    log "k8s8 done"
}

# stop k3s, clear kubelet's CPU manager state and start it again, waiting up to 2 minutes
# for kubelet to write the new policy
k3s_restart() {
    sudo systemctl stop k3s
    # kubelet will not start with a CPU manager state left by another policy
    sudo rm -f /var/lib/kubelet/cpu_manager_state
    sudo systemctl start k3s || true
    for _ in $(seq 60); do
        if sudo test -s /var/lib/kubelet/cpu_manager_state; then break; fi
        sleep 2
    done
    k3s_ready
}

# k3s and its netfilter rules gone, so later host runs see none of them
k3s_down() {
    sudo /usr/local/bin/k3s-uninstall.sh >"$res/k3s-uninstall.log" 2>&1 || true
    { echo "rules left:"; sudo iptables-save 2>/dev/null | grep -c -i 'kube\|flannel\|cni' || true; } >>"$res/k3s-uninstall.log"
    log "k3s down"
}

# pin_irqs <cpus>: every IRQ to <cpus>. MANA spreads its queues' IRQs over all CPUs again
# whenever Azure re-adds the VF, so this is redone before each cross-host run.
pin_irqs() {
    local irq
    for irq in /proc/irq/[0-9]*; do echo "$1" | sudo tee "$irq/smp_affinity_list" >/dev/null 2>&1 || true; done
}

# the accelerated-networking VF behind eth0 (its netdev has eth0 as master), if any
vf_dev() {
    local i
    for i in /sys/class/net/*; do
        if [[ -e $i/master ]]; then
            basename "$i"
            return
        fi
    done
}

# irq_rcv <on|off>: the MANA queues' IRQs on the receiver's core, or back where they were
irq_rcv() {
    local n a
    if [[ $1 == on ]]; then
        topo8
        awk -F: '/mana/ { gsub(/ /, "", $1); print $1 }' /proc/interrupts | while read -r n; do
            echo "$n $(cat "/proc/irq/$n/smp_affinity_list")"
        done >"$res/mana-irqs.saved"
        while read -r n a; do echo "$xrcv8" | sudo tee "/proc/irq/$n/smp_affinity_list" >/dev/null 2>&1 || true; done <"$res/mana-irqs.saved"
    else
        while read -r n a; do echo "$a" | sudo tee "/proc/irq/$n/smp_affinity_list" >/dev/null 2>&1 || true; done <"$res/mana-irqs.saved"
    fi
}

# busy_net <on|off>: socket busy reads (Aeron's receiver calls recvmmsg directly, so
# net.core.busy_poll, which only poll and select use, would do nothing), with NAPI deferring
# hard IRQs and flushing on a timer on the VF
busy_net() {
    local vf on=$([[ $1 == on ]] && echo 1 || echo 0)
    vf=$(vf_dev)
    sudo sysctl -q -w net.core.busy_read=$((on * 50))
    if [[ -n $vf ]]; then
        echo $((on * 2)) | sudo tee "/sys/class/net/$vf/napi_defer_hard_irqs" >/dev/null 2>&1 || true
        echo $((on * 200000)) | sudo tee "/sys/class/net/$vf/gro_flush_timeout" >/dev/null 2>&1 || true
    fi
}

# jumbo <on|off>: MTU 9000 inside the VNet, as MANA allows there, with the default route kept at
# 1500 so that traffic leaving the VNet (ssh from the operator) still fits
jumbo() {
    local route
    if [[ $1 == on ]]; then
        route=$(ip route show default | head -1)
        echo "$route" >"$res/default-route.saved"
        # shellcheck disable=SC2086 # the route's words are ip's arguments
        sudo ip route replace $route mtu 1500 || true
        sudo ip link set eth0 mtu 9000
    else
        sudo ip link set eth0 mtu 1500
        route=$(cat "$res/default-route.saved")
        # shellcheck disable=SC2086
        sudo ip route replace $route || true
    fi
    ip -br link show eth0 >>"$res/jumbo.log" 2>&1
    ip route show default >>"$res/jumbo.log" 2>&1
}

# VF received and sent packets on eth0, busy-poll receives and datapath switches so far
path_stats() {
    local vf_rx vf_tx bp sw
    vf_rx=$(ethtool -S eth0 2>/dev/null | awk '/vf_rx_packets:/ { print $2; exit }')
    vf_tx=$(ethtool -S eth0 2>/dev/null | awk '/vf_tx_packets:/ { print $2; exit }')
    bp=$(awk '/^TcpExt:/ { if (!h) { for (i = 1; i <= NF; i++) if ($i == "BusyPollRxPackets") c = i; h = 1 } else print $c }' /proc/net/netstat)
    sw=$(sudo dmesg 2>/dev/null | grep -ci 'data path switched' || true)
    echo "${vf_rx:-0} ${vf_tx:-0} ${bp:-0} ${sw:-0}"
}

# the newest Debian kernel, Linux from trixie-backports in its cloud flavour (built for Hyper-V
# and the MANA NIC), then a reboot into it
kernel_latest() {
    echo 'deb http://deb.debian.org/debian trixie-backports main' | sudo tee /etc/apt/sources.list.d/backports.list >/dev/null
    sudo apt-get update -qq
    sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq -t trixie-backports linux-image-cloud-amd64 \
        >"$res/kernel-install.log" 2>&1
    { echo "running: $(uname -r)"; dpkg -l 'linux-image-*' | grep '^ii'; } >>"$res/kernel-install.log"
    sudo systemd-run --on-active=3 /bin/systemctl reboot >/dev/null
    log "rebooting into the backports kernel"
}

# boot8 <kernel arguments...>: reboots into the 8-vCPU isolation (every CPU but CPU 0 kept from
# the scheduler, timer ticks, RCU callbacks and IRQs) plus the arguments given
boot8() {
    topo8
    sudo mkdir -p /etc/default/grub.d
    printf '%s\n' "GRUB_CMDLINE_LINUX_DEFAULT=\"\$GRUB_CMDLINE_LINUX_DEFAULT isolcpus=nohz,domain,managed_irq,$iso8 nohz_full=$iso8 rcu_nocbs=$iso8 irqaffinity=$hk8 $*\"" |
        sudo tee /etc/default/grub.d/99-rusteron-isolation.cfg >/dev/null
    sudo update-grub >"$res/update-grub.log" 2>&1
    grep -c 'isolcpus=' /boot/grub/grub.cfg >>"$res/update-grub.log" 2>&1 || true
    sudo systemd-run --on-active=3 /bin/systemctl reboot >/dev/null
    log "rebooting into isolation of $iso8 $*"
}

# what low-latency boxes add on top of isolation: SMT off, no lockup watchdogs or audit,
# staggered ticks, no transparent huge pages, and idle CPUs polling instead of halting, which
# with SMT off cannot steal from a busy twin
TUNE8_ARGS="nosmt idle=poll rcu_nocb_poll nowatchdog nmi_watchdog=0 nosoftlockup skew_tick=1 transparent_hugepage=never audit=0"

# the runtime half of the tuning, after each boot: every IRQ, kernel workqueue and periodic job
# on CPU 0 or off, and nothing in the background that does not need to run
tune8_runtime() {
    sudo systemctl stop irqbalance unattended-upgrades walinuxagent apt-daily.timer apt-daily-upgrade.timer man-db.timer \
        fstrim.timer e2scrub_all.timer 2>/dev/null || true
    pin_irqs "$hk8"
    echo 1 | sudo tee /sys/devices/virtual/workqueue/cpumask >/dev/null 2>&1 || true
    echo 0 | sudo tee /sys/kernel/mm/ksm/run >/dev/null 2>&1 || true
    sudo sysctl -q -w kernel.watchdog=0 vm.stat_interval=120 kernel.numa_balancing=0 2>/dev/null || true
    # console messages go synchronously to the serial port the image logs to
    sudo sysctl -q -w kernel.printk="3 4 1 3" 2>/dev/null || true
    # as Azure recommends with accelerated networking, so a switch between the VF and the
    # synthetic path drops nothing
    sudo sysctl -q -w net.ipv4.conf.all.rp_filter=2 net.ipv4.conf.default.rp_filter=2 2>/dev/null || true
    sudo swapoff -a || true
}

# --- archive under load -------------------------------------------------------------------

# The local NVMe disk and the Premium SSD v2 data disks, each formatted xfs and mounted at
# /mnt/nvme and /mnt/pv2-<MB/s>. A device is used only if it has no partitions and no mount, and
# it is picked by model: Azure's local NVMe reports "Microsoft NVMe Direct Disk", managed disks
# (the OS disk too) "MSFT NVMe Accelerator", so the data disks are the unpartitioned ones of
# those, each matched by its size to its LAB_DATA_DISK spec for the MB/s it was given.
disks8() {
    local name model dev kind gib spec
    lsblk -o NAME,MODEL,SIZE,TYPE,MOUNTPOINTS >"$res/lsblk.txt" 2>&1
    echo "data disks: ${LAB_DATA_DISK:-none}" >"$res/disks.txt"
    while read -r name; do
        dev=/dev/$name
        model=$(cat "/sys/block/$name/device/model" 2>/dev/null | sed 's/ *$//')
        if [[ $(lsblk -n "$dev" | wc -l) -ne 1 || -n $(lsblk -n -o MOUNTPOINTS "$dev" | tr -d ' \n') ]]; then
            echo "$dev ($model): partitioned or mounted, skipped" >>"$res/disks.txt"
            continue
        fi
        case $model in
            *Direct*) kind=nvme ;;
            *Accelerator*)
                gib=$(($(lsblk -bdn -o SIZE "$dev") >> 30)) kind=
                for spec in ${LAB_DATA_DISK:-}; do
                    IFS=: read -r _ s _ m <<<"$spec"
                    if [[ $s == "$gib" ]]; then kind=pv2-$m; fi
                done
                if [[ -z $kind ]]; then echo "$dev ($model, $gib GiB): no LAB_DATA_DISK spec of that size, skipped" >>"$res/disks.txt"; continue; fi ;;
            *) echo "$dev ($model): unknown model, skipped" >>"$res/disks.txt"; continue ;;
        esac
        if mountpoint -q "/mnt/$kind"; then continue; fi
        sudo mkfs.xfs -q -f "$dev" && sudo mkdir -p "/mnt/$kind" && sudo mount -o noatime "$dev" "/mnt/$kind" &&
            sudo chown "$(id -u):$(id -g)" "/mnt/$kind" && echo "$dev ($model): /mnt/$kind" >>"$res/disks.txt"
    done < <(lsblk -dn -o NAME,TYPE | awk '$2 == "disk" { print $1 }')
    cat "$res/disks.txt"
    df -h $(disk_kinds | sed 's|^|/mnt/|') >>"$res/disks.txt" 2>&1 || true
}

# the disks disks8 mounted, by name: nvme, pv2-<MB/s>...
disk_kinds() {
    local d
    for d in /mnt/nvme /mnt/pv2-*; do
        if mountpoint -q "$d"; then echo "${d#/mnt/}"; fi
    done
}

# dirty <default|large>: the kernel's page-cache write-back limits. large lets 16 GiB of
# unwritten data build up before writers are throttled, and starts write-back at 512 MiB
dirty() {
    case $1 in
        default) sudo sysctl -q -w vm.dirty_ratio=20 vm.dirty_background_ratio=10 ;;
        large) sudo sysctl -q -w vm.dirty_bytes=17179869184 vm.dirty_background_bytes=536870912 ;;
    esac
}

# what each disk does on its own: sequential 1 MiB writes and reads, 4 KiB random writes, and
# writes followed by fdatasync as the archive does at file sync level 1, one fio line each.
# Each job writes a new file, so blocks the disk has never held, unless the label ends in
# -overwrite: then one 8 GiB file is written once untimed and every job runs over it.
# diskbench8 [label, fio by default]
diskbench8() {
    local kind job args out group=${1:-fio}
    for kind in $(disk_kinds); do
        if [[ $group == *-overwrite ]]; then
            fio --name=fill --filename="/mnt/$kind/fio.dat" --size=8G --ioengine=libaio --rw=write --bs=1M --iodepth=32 \
                --direct=1 --output-format=json >"$res/$group-$kind-fill.json" 2>>"$res/fio-errors.log" || true
        fi
        for job in seqwrite-qd1 seqwrite-qd32 seqread-qd32 randwrite4k-qd32 syncwrite64k-qd1 syncwrite1m-qd1; do
            case $job in
                seqwrite-qd1) args=(--rw=write --bs=1M --iodepth=1 --direct=1) ;;
                seqwrite-qd32) args=(--rw=write --bs=1M --iodepth=32 --direct=1) ;;
                seqread-qd32) args=(--rw=read --bs=1M --iodepth=32 --direct=1) ;;
                randwrite4k-qd32) args=(--rw=randwrite --bs=4k --iodepth=32 --direct=1) ;;
                syncwrite64k-qd1) args=(--rw=write --bs=64k --iodepth=1 --fdatasync=1) ;;
                syncwrite1m-qd1) args=(--rw=write --bs=1M --iodepth=1 --fdatasync=1) ;;
            esac
            out=$res/$group-$kind-$job.json
            fio --name="$job" --filename="/mnt/$kind/fio.dat" --size=8G --ioengine=libaio --time_based --runtime=30 \
                --group_reporting --output-format=json "${args[@]}" >"$out" 2>>"$res/fio-errors.log" || true
            python3 - "$out" "$host" "$kind" "$job" "$group" <<'PY' | tee -a "$res/bench.csv"
import json, sys
out, host, kind, job, group = sys.argv[1:]
try:
    j = json.load(open(out))["jobs"][0]
except Exception as e:
    print(f"{host},{group},{kind},{job},error"); sys.exit()
side = "read" if j["read"]["io_bytes"] > j["write"]["io_bytes"] else "write"
d = j[side]
p = d.get("clat_ns", {}).get("percentile", {})
sync = j.get("sync", {}).get("lat_ns", {}).get("percentile", {})
q = lambda m, k: round(m.get(k, 0) / 1000, 1)
print(f"{host},{group},{kind},{job},{d['bw_bytes'] / 1e6:.0f},{d['iops']:.0f},{q(p, '50.000000')},{q(p, '99.000000')},{q(p, '99.900000')},{q(sync, '50.000000')},{q(sync, '99.000000')}")
PY
            if [[ $group != *-overwrite ]]; then rm -f "/mnt/$kind/fio.dat"; fi
        done
        rm -f "/mnt/$kind/fio.dat"
    done
    log "diskbench8 $group done"
}

# The archive host's C driver and a Java Archive attached to it, with its directory on
# /mnt/<disk>. For UDP (a control host given) the driver's sender and receiver spin, pinned to
# cores of their own; for IPC they keep their default idle, leaving those cores to publishers.
# The archive's recorder and replayer get a core's two threads; the rest of its JVM shares CPU
# 0's core. The driver's dir and both pids go to $res/arcd.state for later commands.
# arcd_start <nvme|pv2> <file sync level> [archive threading mode] [control host]
arcd_start() {
    local kind=$1 sync=$2 mode=${3:-DEDICATED} ctl=${4:-} jar net=()
    jar=$lab/rusteron/rusteron-archive/aeron/aeron-all/build/libs/aeron-all-1.52.2.jar
    topo8
    runs=$((runs + 1))
    arc_dir=$shm/x86lab-arcd-$runs archive_dir=/mnt/$kind/archive-$runs
    mkdir -p "$archive_dir"
    if [[ -n $ctl ]]; then
        net=(AERON_SENDER_IDLE_STRATEGY=noop AERON_RECEIVER_IDLE_STRATEGY=noop
            AERON_SENDER_CPU_AFFINITY="$xsnd8" AERON_RECEIVER_CPU_AFFINITY="$xrcv8")
    fi
    env AERON_DIR="$arc_dir" AERON_DIR_DELETE_ON_START=true AERON_DIR_DELETE_ON_SHUTDOWN=true \
        AERON_TERM_BUFFER_SPARSE_FILE=false AERON_CONDUCTOR_CPU_AFFINITY="$first_hk8" \
        AERON_SOCKET_SO_SNDBUF=2097152 AERON_SOCKET_SO_RCVBUF=2097152 AERON_RCV_INITIAL_WINDOW_LENGTH=2097152 \
        ${net[@]+"${net[@]}"} taskset -c "$hk8" "$bin/media_driver" >"$res/arcd-driver.log" 2>&1 &
    arcd_driver=$!
    for _ in $(seq 100); do
        if [[ -e $arc_dir/cnc.dat ]]; then break; fi
        sleep 0.1
    done
    sleep 0.5
    taskset -c "$hk8,$((first_hk8 + 1))" java -Xms2g -Xmx2g -XX:+AlwaysPreTouch -XX:+UseParallelGC -XX:-UsePerfData \
        -XX:+UnlockDiagnosticVMOptions -XX:GuaranteedSafepointInterval=300000 --add-opens java.base/jdk.internal.misc=ALL-UNNAMED \
        -Dagrona.disable.bounds.checks=true -Daeron.dir="$arc_dir" -Daeron.archive.dir="$archive_dir" \
        -Daeron.archive.threading.mode="$mode" -Daeron.archive.file.sync.level="$sync" -Daeron.archive.catalog.file.sync.level="$sync" \
        -Daeron.archive.control.channel="aeron:udp?endpoint=${ctl:-localhost}:8010" "-Daeron.archive.replication.channel=aeron:udp?endpoint=localhost:0" \
        -Daeron.archive.recording.events.enabled=false -Daeron.archive.max.concurrent.recordings=64 \
        -Daeron.archive.max.concurrent.replays=64 -cp "$jar" io.aeron.archive.Archive >"$res/arcd-archive.log" 2>&1 &
    arcd_archive=$!
    sleep 3
    pin_java "$arcd_archive" archive-recorder="$xapp8" archive-replayer="$((xapp8 + 1))" 2>/dev/null ||
        log "arcd: archive threads not pinned (threading $mode)"
    echo "$arc_dir $archive_dir $arcd_driver $arcd_archive" >"$res/arcd.state"
}

# stops what arcd_start started, here or in an earlier command, and removes its archive
arcd_stop() {
    local driver archive
    read -r arc_dir archive_dir driver archive <"$res/arcd.state"
    kill -TERM "$archive" 2>/dev/null || true
    for _ in $(seq 100); do kill -0 "$archive" 2>/dev/null || break; sleep 0.1; done
    kill -INT "$driver" 2>/dev/null || true
    for _ in $(seq 100); do kill -0 "$driver" 2>/dev/null || break; sleep 0.1; done
    rm -rf "$archive_dir"
}

# arcload_run <group> <iostat tag> <arcload args...>: arcload through the archive host's driver,
# with iostat sampling every second alongside; its summary line to bench.csv, all output kept
arcload_run() {
    local group=$1 tag=$2 out line
    shift 2
    out=$res/arcload-$tag.out
    iostat -x -m 1 >"$res/iostat-$tag.txt" 2>&1 &
    local io=$!
    env AERON_DIR="$arc_dir" ARCHIVE_CONTROL="aeron:udp?endpoint=localhost:8010" LABEL="$tag" \
        taskset -c "$hk8" timeout 400 "$bin/impr-ps/arcload" "$@" >"$out" 2>>"$res/arcload-errors.log" || log "arcload $tag failed"
    kill "$io" 2>/dev/null || true
    for line in $(grep -E '^arcload,(record|replay),' "$out"); do echo "$host,$group,local,0,$line" | tee -a "$res/bench.csv"; done
}

# The archive on this host, recording streams published here over IPC: each disk at file sync
# level 0 with 1, 4 and 16 streams of 1 KiB messages flat out for 60 s, and at level 1 with 4;
# then 1, 4 and 16 concurrent replays from disk with the page cache dropped; replays while 4
# streams record; and the archive's SHARED threading against DEDICATED
archload8() {
    topo8
    local pubs=$xsnd8,$((xsnd8 + 1)),$xrcv8,$((xrcv8 + 1)) kind sync n c sync_n writer
    for kind in $(disk_kinds); do
        for sync_n in 0:1 0:4 0:16 1:4; do
            sync=${sync_n%:*} n=${sync_n#*:}
            arcd_start "$kind" "$sync"
            arcload_run "arc-ipc-$kind-sync$sync" "ipc-$kind-s$sync-n$n" record "$n" 1024 60 0 ipc "$pubs"
            arcd_stop
        done
        # replays read back what 16 streams wrote, from the disk rather than the page cache
        arcd_start "$kind" 0
        arcload_run "arc-ipc-$kind-fill" "fill-$kind" record 16 1024 30 0 ipc "$pubs"
        for c in 1 4 16; do
            sync && echo 3 | sudo tee /proc/sys/vm/drop_caches >/dev/null
            arcload_run "arc-replay-$kind" "replay-$kind-c$c" replay "$c" 120 ipc "$pubs"
        done
        # replays of those recordings while 4 new streams record
        sync && echo 3 | sudo tee /proc/sys/vm/drop_caches >/dev/null
        arcload_run "arc-rw-$kind-write" "rw-$kind-write" record 4 1024 60 0 ipc "$xsnd8,$((xsnd8 + 1))" &
        writer=$!
        sleep 5
        arcload_run "arc-rw-$kind-replay" "rw-$kind-replay" replay 4 50 ipc "$xrcv8,$((xrcv8 + 1))"
        # that writer only: a bare wait would also wait for the archive's driver and JVM
        wait "$writer" || true
        arcd_stop
    done
    arcd_start nvme 0 SHARED
    arcload_run "arc-ipc-nvme-shared" "ipc-nvme-shared-n4" record 4 1024 60 0 ipc "$pubs"
    arcd_stop
    log "archload8 done"
}

# Bursts at file sync level 0: on each disk, with the kernel's default write-back limits and with
# large ones, two 30 s bursts of 400 MB/s (4 streams of 1 KiB at 97,656 msgs/s) 60 s apart, then
# the time until the page cache has written everything out. Dirty and Writeback from
# /proc/meminfo are sampled every second throughout.
archburst8() {
    topo8
    local pubs=$xsnd8,$((xsnd8 + 1)),$xrcv8,$((xrcv8 + 1)) kind d tag sampler t
    for kind in $(disk_kinds); do
        for d in default large; do
            tag=burst-$kind-$d
            dirty "$d"
            sync && echo 3 | sudo tee /proc/sys/vm/drop_caches >/dev/null
            arcd_start "$kind" 0
            while :; do
                awk -v t="$(date +%s)" '/^Dirty:/ { d = $2 } /^Writeback:/ { w = $2 } END { printf "%s,%d,%d\n", t, d / 1024, w / 1024 }' /proc/meminfo
                sleep 1
            done >"$res/meminfo-$tag.csv" &
            sampler=$!
            arcload_run "arc-burst-$kind-$d" "$tag-1" record 4 1024 30 97656 ipc "$pubs"
            sleep 60
            arcload_run "arc-burst-$kind-$d" "$tag-2" record 4 1024 30 97656 ipc "$pubs"
            for t in $(seq 600); do
                if awk '/^(Dirty|Writeback):/ { s += $2 } END { exit !(s < 65536) }' /proc/meminfo; then break; fi
                sleep 1
            done
            echo "$host,arc-burst-drain,$kind-$d,0,$t" | tee -a "$res/bench.csv"
            kill "$sampler" 2>/dev/null || true
            arcd_stop
        done
    done
    dirty default
    log "archburst8 done"
}

# the peer's archive host commands, with iostat on the peer for the length of one load
peer() { "${peer_ssh[@]}" "/srv/x86lab/harness/vm.sh $*"; }

# On the publisher host, with the archive on LAB_PEER_IP: streams published here and recorded
# there over UDP (each disk, sync level 0, 4 and 16 streams of 1 KiB flat out for 60 s);
# replays from the peer's disk to here; and the cross-host round trip while 4 streams record at
# 25, 50 and 75% of their measured maximum, which is all that rtt runs. xarchload8 [all|rtt]
xarchload8() {
    local peer_ip=${LAB_PEER_IP:?LAB_PEER_IP: the archive host} self kind sync n c f max line writer what=${1:-all}
    topo8
    self=$(hostname -I | awk '{print $1}')
    peer_ssh=(ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new "$peer_ip")
    local pubs=$((xapp8 + 1)),$((xsnd8 + 1)),$((xrcv8 + 1))
    local env=(AERON_TERM_BUFFER_SPARSE_FILE=false AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true
        AERON_SOCKET_SO_SNDBUF=2097152 AERON_SOCKET_SO_RCVBUF=2097152 AERON_RCV_INITIAL_WINDOW_LENGTH=2097152
        AERON_SENDER_IDLE_STRATEGY=noop AERON_RECEIVER_IDLE_STRATEGY=noop AERON_CONDUCTOR_CPU_AFFINITY="$first_hk8"
        AERON_SENDER_CPU_AFFINITY="$xsnd8" AERON_RECEIVER_CPU_AFFINITY="$xrcv8")
    local control="aeron:udp?endpoint=$peer_ip:8010" response="aeron:udp?endpoint=$self:0"
    # xarc <group> <tag> <arcload args...>: arcload here against the peer's archive, iostat there
    xarc() {
        local group=$1 tag=$2 out
        shift 2
        out=$res/arcload-$tag.out
        peer iostat-up "$tag"
        env "${env[@]}" AERON_DIR="$run_dir" ARCHIVE_CONTROL="$control" ARCHIVE_RESPONSE="$response" LABEL="$tag" \
            taskset -c "$hk8" timeout 400 "$bin/impr-ps/arcload" "$@" >"$out" 2>>"$res/arcload-errors.log" || log "arcload $tag failed"
        peer iostat-down
        grep -E '^arcload,(record|replay),' "$out" | sed "s/^/$host,$group,xhost,0,/" | tee -a "$res/bench.csv" || true
    }
    hk=$hk8 driver_start "${env[@]}"
    for kind in $([[ $what == all ]] && peer disk-kinds); do
        for n in 4 16; do
            peer arcd-up "$kind" 0 DEDICATED "$peer_ip"
            xarc "xarc-$kind-sync0" "x-$kind-s0-n$n" record "$n" 1024 60 0 "udp:$peer_ip:30100" "$pubs"
            peer arcd-down
        done
    done
    # replays from the peer's NVMe to here, page cache dropped first
    if [[ $what == all ]]; then
        peer arcd-up nvme 0 DEDICATED "$peer_ip"
        xarc "xarc-fill" "x-fill" record 16 1024 30 0 "udp:$peer_ip:30100" "$pubs"
        for c in 1 4 16; do
            peer drop-caches
            xarc "xarc-replay" "x-replay-c$c" replay "$c" 120 "udp:$self:31000" "$pubs"
        done
        peer arcd-down
    fi
    # the round trip under a fixed share of the measured maximum load
    peer arcd-up nvme 0 DEDICATED "$peer_ip"
    xarc "xarc-max" "x-max-n4" record 4 1024 30 0 "udp:$peer_ip:30100" "$pubs"
    max=$(awk -F, '/^arcload,record,/ { print $7; exit }' "$res/arcload-x-max-n4.out")
    if ! [[ $max =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
        log "xarchload8: no maximum rate measured, so no round trips under load"
        max=
    fi
    for f in ${max:+25 50 75}; do
        # ports of its own for each share: the last pong, killed, lingers in the archive's driver
        # until its client times out, and its image would otherwise reach this ping
        local ping_ep=$peer_ip:$((20100 + f)) pong_ep=$self:$((20200 + f))
        peer pong-arcd "$ping_ep" "$pong_ep"
        xarc "xarc-load$f" "x-load$f" record 4 1024 40 "$(( ${max%.*} * f / 400 ))" "udp:$peer_ip:30100" "$pubs" &
        writer=$!
        sleep 5
        line=$(env "${env[@]}" AERON_DIR="$run_dir" LABEL="load$f" taskset -c "$hk8" timeout 120 \
            "$bin/impr/rtt" xping "$ping_ep" "$pong_ep" "$UDP_N" "$UDP_W" "$xapp8" 2>>"$res/client-errors.log") ||
            line="error,load$f,xping"
        echo "$host,xarc-rtt-load$f,xhost,0,$line" | tee -a "$res/bench.csv"
        # that loader only: a bare wait would also wait for this host's driver
        wait "$writer" || true
        peer pong-down
    done
    peer arcd-down
    driver_stop
    log "xarchload8 done"
}

# On the archive host: pong through the archive host's driver, for the round trip under load
pong_arcd() {
    local dir
    read -r dir _ <"$res/arcd.state"
    topo8
    setsid env AERON_DIR="$dir" taskset -c "$hk8" "$bin/impr/rtt" xpong "$1" "$2" "$((xsnd8 + 1))" \
        </dev/null >"$res/pong.log" 2>&1 &
    echo $! >"$res/pong.pid"
    : >"$res/pong-driver.pid"
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
    kernel) kernel_latest ;;
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
    bench8) bench8 pinned ;;
    disks8) disks8 ;;
    diskbench8) diskbench8 ;;
    diskbench8-*) diskbench8 "fio-${1#diskbench8-}" ;;
    archload8) archload8 ;;
    archburst8) archburst8 ;;
    disk-kinds) disk_kinds ;;
    xarchload8) xarchload8 ;;
    xarcrtt8) xarchload8 rtt ;;
    arcd-up) shift; arcd_start "$@" ;;
    arcd-down) arcd_stop ;;
    pong-arcd) shift; pong_arcd "$@" ;;
    drop-caches) sync && echo 3 | sudo tee /proc/sys/vm/drop_caches >/dev/null ;;
    iostat-up) setsid iostat -x -m 1 </dev/null >"$res/iostat-$2.txt" 2>&1 & echo $! >"$res/iostat.pid" ;;
    iostat-down) kill "$(cat "$res/iostat.pid")" 2>/dev/null || true ;;
    xhost8-*) xhost8 "${1#xhost8-}" ;;
    xnet8-*) xnet8 "${1#xnet8-}" ;;
    loss) shift; loss "$@" ;;
    loss-count) loss_count ;;
    pong-up8) shift; pong_up8 "$@" ;;
    xsub-up8) shift; xsub_up8 "$@" ;;
    xsub-result) xsub_result ;;
    path-stats) path_stats ;;
    pin-irqs) topo8 && pin_irqs "$hk8" ;;
    busy-net) busy_net "$2" ;;
    irq-rcv) irq_rcv "$2" ;;
    jumbo) jumbo "$2" ;;
    archive8) archive8 ;;
    k8s8) k8s8 ;;
    k3s-down) k3s_down ;;
    isolate8) boot8 ;;
    bench8-isolated) bench8 isolated ;;
    tune8) boot8 "$TUNE8_ARGS" ;;
    bench8-tuned) bench8 tuned ;;
    tune8-nomit) boot8 "$TUNE8_ARGS mitigations=off" ;;
    bench8-tuned-nomit) bench8 tuned-nomit ;;
    pong-up) shift; pong_up "$@" ;;
    pong-down) pong_down ;;
    bench-xhost) bench_xhost ;;
    test) test_phase ;;
    *) echo "usage: $0 bootstrap|build|bench|abudp|bench-pinned|isolate|bench-isolated|test" >&2; exit 2 ;;
esac
