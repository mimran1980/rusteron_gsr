#!/bin/bash
# the pod's app container: ping and pong pinned one each to this container's first two CPUs
set -u
. /lab/k8s-cpu/common.sh
mapfile -t cpus < <(my_cpus)
ping=${cpus[0]} pong=${cpus[1]}
# under the default policy the container may use every CPU: keep off CPU 0 and 1, as on the host
if ((${#cpus[@]} > 2)); then ping=2 pong=3; fi
echo "app cpus: ${cpus[*]}"
throttling app
n=0
for rep in 1 2 3; do
    for t in ipc udp tput; do
        n=$((n + 1))
        dir=/aeron/run-$n
        while [ ! -e "$dir/cnc.dat" ]; do sleep 0.1; done
        sleep 0.5
        case $t in
            ipc) AERON_DIR=$dir LABEL=h-ipc timeout 120 /lab/rtt ipc 2000000 200000 "$ping" "$pong" ;;
            udp) AERON_DIR=$dir LABEL=h-udp timeout 120 /lab/rtt udp 300000 50000 "$ping" "$pong" ;;
            tput) AERON_DIR=$dir LABEL=h-tput timeout 60 /lab/tput 5 "$ping" "$pong" ;;
        esac || echo "run failed: $t"
        touch "/aeron/done-$n"
    done
done
throttling app
