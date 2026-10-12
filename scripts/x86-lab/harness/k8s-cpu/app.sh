#!/bin/bash
# the pod's app container: ping and pong pinned one each to a CPU, REPS times ipc and tput
set -u
. /lab/k8s-cpu/common.sh
mapfile -t cpus < <(my_cpus)
has() { [[ " ${cpus[*]} " == *" $1 "* ]]; }
mask=
if [[ ${EXCLUSIVE:-0} == 1 ]]; then
    # CPUs of its own: ping and pong on the first two, the client's other threads on the rest
    ping=${cpus[0]} pong=${cpus[1]}
    if ((${#cpus[@]} > 2)); then mask=$(IFS=,; echo "${cpus[*]:2}"); fi
elif [[ -n ${APP_PING:-} ]]; then
    if has "$APP_PING" && has "$APP_PONG"; then
        ping=$APP_PING pong=$APP_PONG
        if [[ -n ${APP_MASK:-} ]] && has "${APP_MASK%%,*}"; then mask=$APP_MASK; fi
    else
        # a shared pool without them: its last two CPUs, the furthest from CPU 0
        ping=${cpus[${#cpus[@]} - 2]} pong=${cpus[${#cpus[@]} - 1]}
    fi
elif ((${#cpus[@]} > 2)); then
    # the 4-vCPU default-policy pod: keep off CPU 0 and 1, as on the host
    ping=2 pong=3
else
    ping=${cpus[0]} pong=${cpus[1]}
fi
echo "app cpus: ${cpus[*]} ping=$ping pong=$pong others=${mask:-any}"
run() { if [[ -n $mask ]]; then taskset -c "$mask" "$@"; else "$@"; fi; }
throttling app
n=0
for rep in $(seq "$REPS"); do
    for t in ipc tput; do
        n=$((n + 1))
        dir=/aeron/run-$n
        while [ ! -e "$dir/cnc.dat" ]; do sleep 0.1; done
        sleep 0.5
        case $t in
            ipc) AERON_DIR=$dir LABEL=h-ipc run timeout 120 /lab/rtt ipc 2000000 200000 "$ping" "$pong" ;;
            tput) AERON_DIR=$dir LABEL=h-tput run timeout 60 /lab/tput 5 "$ping" "$pong" ;;
        esac || echo "run failed: $t"
        touch "/aeron/done-$n"
    done
done
throttling app
