**Note:** These benchmarks are environment-sensitive; rerun them on your own hardware. Everything below was measured on 2026-10-09 on two Azure `Standard_D8s_v6` VMs (Intel Xeon Platinum 8573C, 4 cores × 2 SMT threads, 32 GiB) in one proximity placement group in North Central US, with accelerated networking (the MANA NIC). They ran Debian 13 with Linux 7.2.6 (the `trixie-backports` cloud kernel), Aeron 1.52.2 and rustc 1.95.0. UDP is measured between the two VMs, never over loopback.

# Intel D8s_v6 pair on Azure (2026-10-09)

## Summary

- **UDP between the hosts:** a 32-byte round trip took 38 µs at p50 with a pinned, dedicated driver on the stock kernel. With CPU isolation and `net.core.busy_read`, it took 31 µs at p50 and 46 µs at p99.99, against 157 µs without busy polling.
  - Putting the NIC's interrupts on the receiver's core did nearly as well (p99.99 49–57 µs). With the kernel tuned it gave the lowest max: 74 µs (62–84).
  - Without one of these two, p99.99 stayed at 120–160 µs in every kernel state. The tail is interrupt handling, not CPU placement.
- **UDP throughput** (32-byte messages):
  - 2 MiB socket buffers and receiver window: 13 M msgs/s. Aeron's default 128 KiB gives 1.9 M.
  - MTU 9000, with Aeron's MTU at 8192: 28.5 M msgs/s.
  - Socket sizes made no difference to round-trip latency, with one message in flight.
- **IPC on one host:** about 0.31 µs at p50 whatever is pinned.
  - Isolation cut p99.99 from 2.5 to 0.8–1.0 µs, and most reps' max to 4–7 µs.
  - The client's other threads (its conductor) must stay off the ping and pong CPUs. On them, p99.99 stayed at 3.1–3.6 µs.
- **Kernel tuning beyond isolation:**
  - SMT off, `idle=poll`, no watchdogs and the rest of the tuned state took 3.7 µs off the cross-host p50 (38.6 → 34.9 µs). IPC didn't change.
  - `mitigations=off` took off another 1 µs and lowered p99.99 from 156 to 121 µs, at a security cost.
- **Java archive:** recording to tmpfs ran at 6.2 M msgs/s (1.6 GB/s, 256-byte messages). Confining the whole JVM to the housekeeping CPU cost 25%. A recorded IPC ping-pong sometimes stalls (see [Open issues](#open-issues)).

## Method

- **Machines.** Both VMs are created and deleted by `scripts/x86-lab/lab.sh` with `LAB_PAIR=1 LAB_LOCKSTEP=1`. Each phase runs on both VMs and finishes on both before the next starts, so both hosts are always in the same kernel state.
  - The first VM (intel-a) pings and the second (intel-b) pongs.
  - SMT twins are adjacent: CPUs 0–1, 2–3, 4–5, 6–7.
- **CPU layout.**
  - CPU 0 does housekeeping alone: interrupts, the drivers' conductors and the clients' other threads.
  - Every busy-spinning thread is pinned to its own core.
  - On one host (IPC): ping on CPU 2, pong on 4; a SHARED driver, where tested, on 6.
  - Across hosts: the app thread (ping or pong) on 2, the driver's sender on 4 and its receiver on 6. The driver pins its own threads (`AERON_CONDUCTOR_CPU_AFFINITY` and the like).
- **Kernel states**, applied one after the other to both VMs:

| State | What it adds |
|---|---|
| pinned | the stock kernel, with the pinning above |
| isolated | `isolcpus=nohz,domain,managed_irq,1-7 nohz_full=1-7 rcu_nocbs=1-7 irqaffinity=0` |
| tuned | `nosmt idle=poll rcu_nocb_poll nowatchdog nmi_watchdog=0 nosoftlockup skew_tick=1 transparent_hugepage=never audit=0`. At boot: every IRQ and the kernel workqueues on CPU 0 (re-pinned before each cross-host run); `irqbalance`, the Azure agent and the apt, man-db, fstrim and e2scrub timers stopped; KSM off; `kernel.watchdog=0`, `vm.stat_interval=120`, `kernel.numa_balancing=0`, a quiet `kernel.printk`, `rp_filter=2`; swap off. SMT off leaves CPUs 0, 2, 4 and 6. |
| tuned-nomit | tuned, plus `mitigations=off` (the CPU vulnerability files then read "Vulnerable") |

- **Aeron settings throughout.**
  - Non-sparse term buffers (`AERON_TERM_BUFFER_SPARSE_FILE=false`) and pre-touched client mappings (`AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY=true`).
  - `AERON_DIR` on 2 MiB `hugetlbfs` (`AERON_FILE_PAGE_SIZE=2097152`).
  - Across hosts, the socket profile of Adaptive's low-latency driver: `AERON_SOCKET_SO_SNDBUF`, `AERON_SOCKET_SO_RCVBUF` and `AERON_RCV_INITIAL_WINDOW_LENGTH` all 2 MiB, with `net.core.rmem_max` and `wmem_max` at 16 MiB. The "defaults" rows keep Aeron's own.
  - The dedicated driver's sender and receiver use `noop` idle; its conductor keeps the default backoff.
  - Exclusive publications; pong echoes with `try_claim`.
- **Workloads.**
  - Round trip: 32-byte messages, one in flight. IPC does 2,000,000 round trips after 200,000 of warm-up; UDP does 300,000 after 50,000. Each run stops after 20 s.
  - Throughput: 32-byte messages offered flat out. The figure is the median of five one-second samples after a one-second warm-up.
  - Each state runs 5 reps, with the variants in a different order each rep.
- **Reading the tables.**
  - Each cell is the median across reps, with the range across reps in brackets.
  - A difference counts only when the ranges do not overlap.
  - Max is the slowest single round trip of a rep, one sample, so it varies most.
- **Validity checks.**
  - Azure can move a VM's traffic from the NIC's direct path (the VF) to its software path during host servicing. Every cross-host run recorded both hosts' VF packet counters and any "data path switched" messages. All 200 runs stayed on the VF.
  - The busy-polling runs recorded the kernel's busy-poll receive counter: about 180,000–270,000 per run on each host, so busy polling did run.
  - Azure's scheduled-events service was queried at the start of each phase: 24 queries listed no events, and 20 returned nothing.
  - The clock source was `tsc` throughout.

## UDP round trip between the two hosts (µs)

Variants:
- **dedicated**: sender and receiver threads, each pinned with `noop` idle.
- **defaults**: the same, with Aeron's default socket buffers and window.
- **SHARED_NETWORK**: one `noop` thread does sending and receiving; the conductor stays on CPU 0.
- **SHARED**: the whole driver is one `noop` thread on CPU 4.
- **busy_read**: dedicated, with `net.core.busy_read=50`, and on the VF `napi_defer_hard_irqs=2` and `gro_flush_timeout=200000`, on both hosts.
- **IRQs on receiver core**: dedicated, with the NIC queues' IRQs on CPU 6, where the receiver spins.

| state, variant | reps | p50 | p99 | p99.9 | p99.99 | max |
|---|---|---|---|---|---|---|
| pinned, dedicated | 5 | 38.34 (33.12–41.57) | 41.50 (35.84–58.05) | 52.51 (44.06–65.98) | 132.48 (117.12–156.93) | 420.86 (324.35–581.12) |
| pinned, defaults | 5 | 38.17 (36.58–43.65) | 50.02 (40.96–51.26) | 53.95 (50.85–61.44) | 124.22 (71.68–164.35) | 479.74 (179.20–872.45) |
| pinned, SHARED_NETWORK | 5 | 40.73 (34.98–42.14) | 45.41 (37.50–56.22) | 55.30 (47.39–62.02) | 130.30 (125.50–147.46) | 349.44 (273.66–378.62) |
| pinned, SHARED | 5 | 39.74 (37.28–41.98) | 45.09 (43.33–54.91) | 57.09 (54.37–62.69) | 160.64 (158.34–179.97) | 655.87 (462.85–1256.45) |
| pinned, busy_read | 5 | 33.09 (31.07–34.62) | 39.30 (34.56–49.41) | 43.84 (40.93–63.42) | 147.07 (115.26–170.37) | 430.08 (275.20–607.74) |
| pinned, IRQs on receiver core | 5 | 33.34 (33.31–33.38) | 35.71 (35.55–36.06) | 44.73 (44.00–45.76) | 120.45 (116.86–150.14) | 726.01 (284.93–988.67) |
| isolated, dedicated | 5 | 38.62 (38.40–38.94) | 51.30 (50.88–53.09) | 61.09 (59.90–64.48) | 157.18 (146.69–161.79) | 575.49 (338.69–1597.44) |
| isolated, defaults | 5 | 38.66 (38.21–38.66) | 51.45 (51.04–51.62) | 60.67 (60.38–62.72) | 157.31 (149.89–170.50) | 449.79 (310.01–683.01) |
| isolated, SHARED_NETWORK | 5 | 39.81 (39.30–39.97) | 53.31 (53.02–53.76) | 63.65 (62.85–64.29) | 157.95 (156.41–178.56) | 761.34 (647.17–1935.36) |
| isolated, SHARED | 5 | 40.77 (39.90–40.80) | 46.37 (45.53–46.62) | 53.66 (53.12–55.23) | 156.93 (151.42–166.91) | 472.83 (333.82–919.55) |
| isolated, busy_read | 5 | 31.05 (30.77–31.87) | 34.05 (33.44–35.30) | 37.60 (36.06–38.46) | 45.98 (42.24–49.34) | 230.40 (179.71–343.04) |
| isolated, IRQs on receiver core | 5 | 34.17 (33.82–34.17) | 37.15 (36.93–38.14) | 43.20 (41.82–54.14) | 56.58 (52.48–81.22) | 173.82 (160.51–1132.54) |
| tuned, dedicated | 5 | 34.88 (34.72–35.04) | 39.36 (39.04–39.49) | 43.45 (42.91–43.94) | 156.29 (150.53–162.30) | 522.75 (481.02–1098.75) |
| tuned, defaults | 5 | 34.98 (34.81–35.07) | 39.30 (39.26–39.30) | 42.88 (42.72–43.23) | 147.20 (142.97–156.93) | 497.15 (431.36–532.48) |
| tuned, SHARED_NETWORK | 5 | 35.62 (35.62–35.65) | 39.68 (39.62–39.84) | 44.13 (43.84–44.35) | 149.76 (142.59–158.21) | 577.53 (490.75–1926.14) |
| tuned, SHARED | 5 | 34.91 (34.85–34.98) | 38.27 (38.14–38.46) | 44.29 (44.09–44.35) | 153.34 (147.46–166.27) | 500.99 (475.65–748.03) |
| tuned, busy_read | 5 | 31.58 (30.93–32.45) | 35.01 (33.70–37.76) | 38.56 (37.05–40.90) | 43.90 (43.42–52.99) | 120.13 (89.41–830.98) |
| tuned, IRQs on receiver core | 5 | 34.21 (34.14–34.30) | 36.73 (36.58–36.86) | 42.11 (41.63–42.46) | 51.77 (51.07–53.02) | 74.24 (62.02–83.52) |
| tuned-nomit, dedicated | 5 | 33.85 (33.53–33.95) | 38.24 (37.98–38.37) | 43.10 (42.08–59.36) | 120.58 (113.47–123.97) | 172.41 (159.87–299.26) |
| tuned-nomit, defaults | 5 | 33.92 (33.57–34.14) | 38.30 (37.89–38.53) | 41.79 (40.67–42.21) | 125.25 (119.55–135.04) | 285.95 (160.77–677.38) |
| tuned-nomit, SHARED_NETWORK | 5 | 34.88 (34.53–34.98) | 38.59 (38.02–39.42) | 42.81 (42.08–44.16) | 130.75 (126.85–138.24) | 432.89 (185.47–1793.02) |
| tuned-nomit, SHARED | 5 | 34.05 (33.47–34.11) | 37.18 (36.35–39.68) | 42.59 (42.11–45.02) | 126.85 (124.86–139.01) | 183.94 (157.82–489.47) |
| tuned-nomit, busy_read | 5 | 31.01 (30.86–31.05) | 34.05 (33.82–34.62) | 36.83 (36.54–38.40) | 49.89 (41.47–57.53) | 406.78 (57.85–509.69) |
| tuned-nomit, IRQs on receiver core | 5 | 32.99 (32.75–33.09) | 35.71 (34.91–36.64) | 39.62 (39.36–41.28) | 49.25 (48.06–49.76) | 72.51 (67.20–79.36) |

- **Busy polling** (`busy_read`): on isolated cores, against dedicated without it, p50 fell from about 38 to 31 µs and p99.99 from about 157 to 44–50 µs. On the stock kernel the ranges overlap.
- **NIC IRQs on the receiver core:**
  - With isolation, p99.99 fell to 49–57 µs.
  - In the tuned states it gave the lowest max, 72–74 µs; dedicated alone gave 172–523 µs.
  - Against busy polling, the two can't be ranked: their p99.99 and max ranges overlap.
- **The tuned state** cut the dedicated driver's p50 from 38.6 to 34.9 µs. **`mitigations=off`** cut it to 33.9 µs and its p99.99 from 156 to 121 µs.
- **Driver threading:** SHARED, SHARED_NETWORK and dedicated were within about 2 µs of each other at p50 in every state.
- **Socket buffer size:** no effect on round-trip latency.
- **The lowest p50** was about 31 µs per round trip, with this NIC and the kernel network stack.

## UDP throughput between the two hosts (M msgs/s, 32-byte messages)

Publisher on intel-a, subscriber on intel-b, both drivers dedicated with pinned `noop` sender and receiver.

| variant | pinned | isolated | tuned | tuned-nomit |
|---|---|---|---|---|
| 2 MiB socket buffers and window | 13.03 (9.48–13.63) | 13.00 (12.66–13.05) | 12.66 (12.59–12.71) | 14.05 (13.98–14.47) |
| Aeron's defaults (128 KiB receive buffer and window) | 1.94 (1.94–1.94) | 1.93 (1.93–1.93) | 1.93 (1.93–1.94) | 1.94 (1.94–1.94) |
| 2 MiB, MTU 9000 with `AERON_MTU_LENGTH=8192` | 28.48 (23.77–28.77) | 28.47 (28.16–28.75) | 28.73 (28.62–28.90) | 28.77 (28.44–28.80) |
| 2 MiB, 16-message io vectors and sends | 13.65 (9.46–14.32) | 13.89 (13.43–13.96) | 13.36 (13.26–13.39) | 14.79 (14.74–15.12) |

- **Receiver window:** with Aeron's defaults, the 128 KiB window caps a 32-byte stream at 1.9 M msgs/s. 2 MiB raised it 6.7×.
- **Jumbo frames** doubled throughput again.
  - Azure allows MTU 9000 only inside a VNet and directly peered VNets.
  - The run kept the default route at 1500, so traffic leaving the VNet still fit: `ip route replace <default route> mtu 1500`, then `ip link set eth0 mtu 9000`.
- **Batching:** 16-message io vectors and sends (`AERON_SENDER_IO_VECTOR_CAPACITY`, `AERON_RECEIVER_IO_VECTOR_CAPACITY`, `AERON_NETWORK_PUBLICATION_MAX_MESSAGES_PER_SEND`, default 4) added 5–7% in the isolated and tuned states. On the stock kernel the ranges overlap.

## IPC on one host (µs, M msgs/s)

The driver is dedicated with its default idle strategies, so no unpinned thread spins, except in the SHARED rows, where a `noop` SHARED driver has CPU 6 to itself. Rows are given per VM. The isolated, tuned and tuned-nomit states run only the variants that pin every busy thread: the kernel keeps unpinned work off isolated CPUs, so it would crowd onto CPU 0.

| state, variant | intel-a p50 | p99.99 | max | M msgs/s | intel-b p50 | p99.99 | max | M msgs/s |
|---|---|---|---|---|---|---|---|---|
| pinned, nothing pinned | 0.310 | 8.33 (2.27–9.22) | 43.55 (24.66–293.38) | 74.5 | 0.311 | 8.78 (3.10–9.34) | 157.95 (51.74–664.58) | 67.2 |
| pinned, ping and pong pinned | 0.307 | 2.45 (2.28–2.62) | 38.24 (20.54–67.07) | 79.2 | 0.312 | 2.75 (2.63–3.12) | 48.77 (28.37–60.00) | 68.2 |
| pinned, client threads on ping and pong CPUs | 0.307 | 7.34 (6.60–7.59) | 38.14 (21.42–52.70) | 76.5 | 0.312 | 7.71 (7.04–7.77) | 36.41 (30.69–62.91) | 68.4 |
| pinned, SHARED driver | 0.373 | 6.16 (4.42–7.30) | 43.81 (27.07–69.25) | 43.5 | 0.404 | 7.08 (6.77–7.67) | 54.69 (32.83–61.70) | 38.9 |
| isolated, ping and pong pinned | 0.307 | 0.804 (0.730–0.811) | 4.51 (4.23–55.13) | 78.7 | 0.300 | 1.04 (0.988–1.09) | 6.63 (4.95–63.36) | 69.4 |
| isolated, client threads on ping and pong CPUs | 0.306 | 3.29 (2.96–4.30) | 28.40 (22.96–65.79) | 79.0 | 0.304 | 3.10 (3.01–3.82) | 24.43 (21.63–31.92) | 68.1 |
| isolated, SHARED driver | 0.356 | 1.24 (1.13–1.31) | 4.72 (4.16–63.10) | 42.1 | 0.396 | 1.43 (1.35–20.67) | 5.59 (5.48–60.03) | 38.7 |
| tuned, ping and pong pinned | 0.315 | 0.732 (0.647–0.849) | 5.21 (4.71–5.32) | 70.1 | 0.311 | 0.974 (0.801–1.04) | 4.73 (4.65–68.61) | 70.0 |
| tuned, client threads on ping and pong CPUs | 0.315 | 3.21 (3.08–4.84) | 27.18 (21.49–31.65) | 70.3 | 0.311 | 3.35 (3.29–4.25) | 23.95 (16.35–26.06) | 72.7 |
| tuned, SHARED driver | 0.367 | 1.25 (1.15–1.66) | 5.26 (5.03–57.95) | 39.5 | 0.416 | 1.24 (1.16–1.34) | 5.12 (4.88–39.84) | 38.8 |
| tuned-nomit, ping and pong pinned | 0.315 | 0.686 (0.644–1.06) | 4.41 (4.11–65.28) | 73.7 | 0.299 | 1.06 (0.876–1.17) | 4.47 (4.32–16.17) | 71.7 |
| tuned-nomit, client threads on ping and pong CPUs | 0.316 | 3.46 (3.06–4.34) | 20.96 (18.14–59.97) | 76.0 | 0.302 | 3.56 (3.09–3.71) | 20.83 (17.31–58.49) | 68.9 |
| tuned-nomit, SHARED driver | 0.387 | 1.20 (1.16–1.30) | 30.41 (5.12–67.71) | 40.8 | 0.371 | 1.39 (1.10–1.44) | 14.17 (4.42–69.06) | 42.4 |

Across every rep without the SHARED driver, p50 was 0.297–0.317 µs, p99 0.35–0.41 µs and p99.9 0.43–0.54 µs. With it, p50 was 0.33–0.43 µs, p99 0.44–0.63 µs and p99.9 0.47–1.45 µs. The full tables, with every percentile and range, are in the run's results folder (see [How to run](#how-to-run)).

- **The median** didn't move with pinning or kernel state.
- **Isolation** cut p99.99 from 2.5–2.8 to 0.8–1.0 µs on both VMs. Most isolated reps had a max of 4–7 µs, against 20–67 µs pinned; the occasional 55–65 µs outlier keeps the max ranges overlapping.
- **The client's other threads**, its conductor among them, held p99.99 at 3.1–3.6 µs when they ran on the ping and pong CPUs, in every isolated or tuned state. Give them a CPU of their own.
- **A SHARED driver costs IPC.** Its single spinning thread kept p50 at 0.36–0.42 µs and halved throughput.
- **Tuning beyond isolation**, `mitigations=off` included, made no measurable difference to IPC.

## Java archive (intel-a)

An `ArchivingMediaDriver` (Java 21) runs with `-Xms1g -Xmx1g -XX:+AlwaysPreTouch -XX:+UseParallelGC -XX:-UsePerfData -XX:GuaranteedSafepointInterval=300000`, DEDICATED archive threading, and `AERON_DIR` and the archive directory on tmpfs, so that disk speed doesn't hide the CPUs. The recording test records 1,000,000 messages of 256 bytes over IPC, published from a thread pinned to CPU 2. The round-trip test is the IPC ping-pong above, with the ping stream recorded.

| JVM | reps | published (M msgs/s) | recorded (M msgs/s) | recorded MB/s | recorded-ping round trip p50 / p99 / p99.99 / max (µs) |
|---|---|---|---|---|---|
| unpinned | 5 | 6.72 (6.50–6.90) | 6.13 (5.88–6.24) | 1569 | stalled in 5 of 5 |
| on CPU 0 | 5 | 5.11 (4.83–5.21) | 4.62 (4.46–4.73) | 1182 | 0.317 / 1.90 / 8.78 / 38.1 |
| on CPU 0, archive-recorder thread on CPU 4 | 5 | 6.83 (6.59–6.93) | 6.23 (6.00–6.27) | 1595 | 0.322 / 1.93 / 9.10 / 43.4 (1 of 5 stalled) |
| as above, recorder `noop` idle | 5 | 6.86 (6.75–6.90) | 6.23 (6.16–6.27) | 1594 | 0.322 / 1.93 / 11.0 / 22.9 (2 of 5 stalled) |

- **Recording costs IPC latency:** IPC p99 rose from 0.37 µs to about 1.9 µs, because the archive is a second subscriber on the ping stream.
- **Placement:** confining the whole JVM to the housekeeping CPU cut recording throughput by 25%; pinning its recorder to a core of its own got it back. A busy-spinning recorder added nothing.

## Kubernetes pods (single VM, earlier run, 2026-10-09)

These pods ran in an earlier run on one `Standard_D8s_v6`, on Linux 6.12 with k3s v1.36.5, measuring IPC only.
- **Pod shape:** each pod had a driver container (a SHARED `noop` driver, or dedicated in `st-d3-a3`) and an app container (ping and pong), sharing a `HugePages-2Mi` `emptyDir`.
- **Policies:** "def" pods ran under kubelet's default CPU manager; "st" pods ran under the static policy with `reserved-cpus=0`.
- **Caveat:** in `st-d3-a3` the driver's threads were not pinned within its 3 CPUs. rusteron's `media_driver` didn't then apply `AERON_*_CPU_AFFINITY`; it does now.

| pod | CPU requests, driver / app | throttled periods, driver / app | ping, pong CPUs | p50 (µs) | p99.99 (µs) | max (µs) | M msgs/s |
|---|---|---|---|---|---|---|---|
| def-s1-a2 | 1 / 2 | 133 of 571 / 28 of 512 | 2, 4 | 0.372 | 9.35 | 1797 (30–1812) | 38.6 |
| def-s2-a3 | 2 / 3 | 0 / 0 | 2, 4 | 0.382 | 9.06 | 41.0 (32–65) | 34.8 |
| def-s1-a2.5 | 1 / 2.5 | 320 of 581 / 0 | 2, 4 | 0.375 | 9.51 | 48.3 (32–61) | 36.0 |
| def-s1-a2-burst (no CPU limits) | 1 / 2 | no quota | 2, 4 | 0.383 | 9.70 | 33.8 (30–69) | 39.6 |
| st-s1-a2 | 1 / 2 | no quota | 2, 3 (one core's twins) | 0.287 | 8.00 | 19.2 (19–21) | 50.9 |
| st-s2-a3 | 2 / 3 | no quota | 1, 4 | 0.360 | 9.57 | 55.1 (27–62) | 39.2 |
| st-d3-a3 | 3 / 3 | no quota | 4, 5 (one core's twins) | 0.115 | 0.405 | 52.4 (16–84) | 62.5 |
| st-s1-a2.5 | 1 / 2.5 (shared pool) | no quota | 2, 4 | 0.354 | 8.95 | 57.4 (30–68) | 36.3 |

- **The default policy's quota:** under it, a CPU limit is a quota per 100 ms. A driver container with a 1-CPU limit and one spinning thread was throttled in 23–55% of periods: a SHARED driver is more than one thread. Its round trips stalled for up to 1.8 ms. One CPU of headroom (`def-s2-a3`), or no CPU limit (Burstable), removed the throttling.
- **The static policy** gave each Guaranteed container whole CPUs to itself, with no quota at all (`cpu.max` read `max`). Kubernetes 1.36 drops the quota for exclusive CPUs.
- **Medians follow ping and pong's placement:** kubelet put them on one core's two SMT twins in `st-s1-a2` and `st-d3-a3`, the fastest IPC layout. The pods weren't isolated, so p99.99 stayed near 9 µs.

### Configuring a pod

- **kubelet:** `cpu-manager-policy=static` and `reserved-cpus=0`. On k3s these go in `/etc/rancher/k3s/config.yaml` as `kubelet-arg`. kubelet refuses to start with a state file from another policy, so stop k3s, delete `/var/lib/kubelet/cpu_manager_state`, then start it.
- **Room for the pod:** other pods' CPU requests count against the allocatable CPUs. CoreDNS and local-path-provisioner were scaled to zero.
- **Huge pages:** reserve them on the node before kubelet starts (`vm.nr_hugepages` in `/etc/sysctl.d`), or kubelet does not report them. Huge page requests must equal limits. kubelet mounts the volume without `size=`, so the driver needs `AERON_PERFORM_STORAGE_CHECKS=false`.
- **Containers:** make every one Guaranteed with whole CPUs, with one spinning thread per CPU, each pinned inside the container. Give the client's other threads one more CPU: every thread in a container shares its CPUs. The driver and its clients share one `emptyDir`, which every container in the pod can mount.

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: aeron-app
spec:
  containers:
    - name: driver
      image: debian:trixie-slim
      env:
        - {name: AERON_DIR, value: /aeron/driver}
        - {name: AERON_THREADING_MODE, value: SHARED}
        - {name: AERON_SHARED_IDLE_STRATEGY, value: noop}
        - {name: AERON_FILE_PAGE_SIZE, value: "2097152"}
        - {name: AERON_PERFORM_STORAGE_CHECKS, value: "false"}  # kubelet mounts hugetlbfs without size=
        - {name: AERON_TERM_BUFFER_SPARSE_FILE, value: "false"}
      resources:
        requests: {cpu: "2", memory: 1Gi, hugepages-2Mi: 1Gi}
        limits: {cpu: "2", memory: 1Gi, hugepages-2Mi: 1Gi}
      volumeMounts:
        - {name: aeron, mountPath: /aeron}
    - name: app
      image: debian:trixie-slim
      env:
        - {name: AERON_DIR, value: /aeron/driver}
        - {name: AERON_CLIENT_PRE_TOUCH_MAPPED_MEMORY, value: "true"}
      resources:
        requests: {cpu: "3", memory: 1Gi, hugepages-2Mi: 1Gi}
        limits: {cpu: "3", memory: 1Gi, hugepages-2Mi: 1Gi}
      volumeMounts:
        - {name: aeron, mountPath: /aeron}
  volumes:
    - name: aeron
      emptyDir: {medium: HugePages-2Mi}
```

## What else was checked

**No effect inside an Azure Hyper-V guest, or not used in Aeron's path:**
- Interrupt coalescing (`ethtool -C`): MANA has no coalescing control.
- GRO and LRO: plain UDP sockets are never merged.
- `net.core.busy_poll`: Aeron's receiver calls `recvmmsg` directly, never `poll`, so only `busy_read` matters.
- Limiting C-states and `haltpoll`: `idle=poll` bypasses cpuidle, and `haltpoll` needs KVM.
- `tsc=reliable`: Hyper-V already marks the TSC reliable, and `tsc` was the clock source.
- `mlockall`: there is no swap, and Aeron's memory is pre-touched.

**Not done, with the reason:**
- `SCHED_FIFO` for the busy-spinning threads: it gains nothing on isolated cores and risks starving kernel threads.
- NIC IRQs on a quiet core of their own: once SMT is off, a D8s_v6 has no spare core.
- `SO_BUSY_POLL` and `SO_PREFER_BUSY_POLL` per socket: Aeron's C code never sets them.
- Kernel bypass (DPDK): open-source Aeron has no DPDK transport.

## Open issues

- **The recorded IPC ping-pong sometimes stalls.**
  - It stalled in every rep with the JVM unpinned, on both VMs, and in 3 of 15 pinned reps on intel-a. A stall stops the run at its timeout.
  - During a stall the archive's and the Java driver's threads were all idle, the recorder included. So the archive had stopped consuming a stream it was recording, while the IPC publication waited on it as a subscriber; it was not short of CPU.
  - Aeron's counters and the archive's error log were not captured in this run; the lab now saves them.
- **Not measured:** AMD; more than one message in flight; publications created after start-up; a Java driver across hosts.

## How to run

```bash
LAB_PAIR=1 LAB_LOCKSTEP=1 \
LAB_NODES="intel-a:northcentralus:Standard_D8s_v6 intel-b:northcentralus:Standard_D8s_v6" \
LAB_ARMS="impr impr-ps" LAB_EXTRAS=samples \
LAB_PHASES="bootstrap kernel build bench8 xhost8-pinned archive8 isolate8 bench8-isolated xhost8-isolated tune8 bench8-tuned xhost8-tuned tune8-nomit bench8-tuned-nomit xhost8-tuned-nomit" \
scripts/x86-lab/lab.sh
```

`lab.sh` refuses to start unless more than 10 USD of the subscription's free credit is left. It deletes the VMs on any exit. Its header lists the options, and the phases are in `scripts/x86-lab/harness/vm.sh`. Results go to `target/x86lab/results/<time>/<vm>/after-<phase>/results/`: `bench.csv`, `xhost8-runs.csv`, and each state's kernel settings and interrupt counts.

These recipes run the samples on one machine without pinning:

```bash
just benchmark-ipc-throughput-java
just run-aeron-media-driver-rust          # terminal 1
just benchmark-ipc-throughput-rust        # terminal 2
just benchmark-embedded-ping-pong-java
just benchmark-embedded-ping-pong-rust    # with the driver from terminal 1
```
