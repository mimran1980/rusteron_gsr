#!/bin/bash
# Runs inside a pod: the harness against a driver in the same container, with AERON_DIR on
# the pod's /aeron volume. Hot threads on CPUs 2 and 3, the driver on 0 and 1.
set -u
export LD_LIBRARY_PATH=/lab/lib
echo "mount: $(grep ' /aeron ' /proc/mounts)"
for rep in 1 2 3; do
    for t in ipc tput; do
        dir=/aeron/run-$rep-$t
        AERON_DIR=$dir AERON_DIR_DELETE_ON_START=true AERON_DIR_DELETE_ON_SHUTDOWN=true \
            taskset -c 0,1 /lab/media_driver >/tmp/driver.log 2>&1 &
        driver=$!
        for _ in $(seq 100); do
            if [ -e "$dir/cnc.dat" ]; then break; fi
            sleep 0.1
        done
        sleep 0.5
        case $t in
            ipc) AERON_DIR=$dir LABEL=h-ipc taskset -c 0,1 timeout 120 /lab/rtt ipc 2000000 200000 2 3 ;;
            tput) AERON_DIR=$dir LABEL=h-tput taskset -c 0,1 timeout 60 /lab/tput 5 2 3 ;;
        esac || { echo "run failed: $t"; tail -5 /tmp/driver.log; }
        kill -INT "$driver"
        wait "$driver"
    done
done
