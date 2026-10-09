# shared by the two containers of the CPU manager pod
export LD_LIBRARY_PATH=/lab/lib
REPS=${REPS:-3}
RUNS=$((REPS * 2))    # REPS reps of ipc and tput

# this container's CPUs, one per line ("2-3" and "1,3" both expand)
my_cpus() {
    local range
    for range in $(taskset -pc $$ | awk -F': ' '{print $2}' | tr ',' ' '); do
        if [[ $range == *-* ]]; then seq "${range%-*}" "${range#*-}"; else echo "$range"; fi
    done
}

# throttle,<container>,<periods>,<throttled periods>,<throttled µs>,<quota period>,<cpuset>
throttling() {
    awk -v c="$1" -v q="$(cat /sys/fs/cgroup/cpu.max)" -v s="$(cat /sys/fs/cgroup/cpuset.cpus.effective)" \
        '/^nr_periods/{p=$2} /^nr_throttled/{t=$2} /^throttled_usec/{u=$2} END{print "throttle," c "," p "," t "," u "," q "," s}' /sys/fs/cgroup/cpu.stat
}
