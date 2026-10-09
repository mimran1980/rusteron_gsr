#!/bin/bash
# the pod's driver container: one fresh media driver per run, each in /aeron/run-<n>, kept
# until the app container marks the run done
set -u
. /lab/k8s-cpu/common.sh
mapfile -t cpus < <(my_cpus)
echo "driver cpus: ${cpus[*]}"
# shared (default): one noop thread for everything. dedicated: noop sender and receiver,
# with conductor, sender and receiver each pinned to one of the container's first 3 CPUs
if [[ ${DRIVER_MODE:-shared} == dedicated ]]; then
    export AERON_THREADING_MODE=DEDICATED AERON_SENDER_IDLE_STRATEGY=noop AERON_RECEIVER_IDLE_STRATEGY=noop
    if ((${#cpus[@]} >= 3)); then
        export AERON_CONDUCTOR_CPU_AFFINITY=${cpus[0]} AERON_SENDER_CPU_AFFINITY=${cpus[1]} AERON_RECEIVER_CPU_AFFINITY=${cpus[2]}
    fi
else
    export AERON_THREADING_MODE=SHARED AERON_SHARED_IDLE_STRATEGY=noop
fi
throttling driver
for n in $(seq "$RUNS"); do
    AERON_DIR=/aeron/run-$n AERON_DIR_DELETE_ON_START=true AERON_DIR_DELETE_ON_SHUTDOWN=true /lab/media_driver >/tmp/driver-$n.log 2>&1 &
    driver=$!
    while [ ! -e "/aeron/done-$n" ]; do sleep 0.2; done
    kill -INT "$driver"
    wait "$driver"
done
throttling driver
