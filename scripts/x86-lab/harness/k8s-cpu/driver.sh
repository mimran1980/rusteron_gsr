#!/bin/bash
# the pod's driver container: one fresh media driver per run, each in /aeron/run-<n>, kept
# until the app container marks the run done
set -u
. /lab/k8s-cpu/common.sh
echo "driver cpus: $(my_cpus | tr '\n' ' ')"
throttling driver
for n in $(seq "$RUNS"); do
    AERON_DIR=/aeron/run-$n AERON_DIR_DELETE_ON_START=true AERON_DIR_DELETE_ON_SHUTDOWN=true /lab/media_driver >/tmp/driver-$n.log 2>&1 &
    driver=$!
    while [ ! -e "/aeron/done-$n" ]; do sleep 0.2; done
    kill -INT "$driver"
    wait "$driver"
done
throttling driver
