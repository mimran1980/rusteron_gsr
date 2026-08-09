# AWS EKS DPDK ENA — benchmark report (plan §12)

Template and gate definitions for the DPDK ENA benchmark. Fill this out from
an acceptance run bundle (`$RUSTERON_ACCEPTANCE_ARTIFACTS/run-<timestamp>/`,
produced by `scripts/aws-dpdk-acceptance.sh bench`) and check the §12.3 gates
before promoting the transport to any cell.

---

## 1. Environment (reproducibility, §12.2)

Capture from the bundle's `env-metadata.json` (the acceptance script writes
it) — never a prose description. Reproduce a result only against the same
shape of inputs.

| Field | Value |
|---|---|
| Cell / AZ | `dpdk-test` (e.g. `us-west-2b`) |
| Node A instance type | `m6i.xlarge` |
| Node A kernel | `6.1.` (AL2023) |
| Node A DPDK | `23.11.1` |
| Node A ENA driver | `2.10.0` |
| Node A kubelet | `1.31.` |
| Node B instance type | (as above) |
| git SHA | `<sha from bundle>` |
| Sizes | 64 / 256 / 1408 / 16384 / 1048576 B |
| Warm-up | 30 s (saturation calibration) |
| Runs per cell | 10 × 60 s |
| Loads | common = 50% of lowest-mode saturation, stress = 70% (same offered rate across all three modes) |

## 2. Methodology (§12.2)

- **Saturation** is calibrated per mode and per size: a `perf` run at
  `RUSTERON_HARNESS_LOAD_RPS=0` (as fast as the mode will go). The **common**
  load shared by all three modes is 50% of the *lowest* mode's saturation at
  each size, so the offered rate is identical across modes — throughput
  differences are then attributable to the transport, not the load generator.
- **Latency** is one-way, measured by the receiving harness role from a
  wall-clock timestamp written into the 16-byte wire header, recorded into an
  HDR histogram and dumped raw to `*.raw` sample files. Reports carry
  `latency_p50_ns` / `latency_p99_ns` / `latency_max_ns`.
- **Metrics** below are the median across the 10 runs of each
  (mode, size, load) cell.
- **Modes**: `kernel` (tuned default UDP over the primary ENA),
  `dpdk-off` (DPDK ENA, ENA Express off), `dpdk-on` (DPDK ENA, ENA Express on —
  only if the ENI supports it).

## 3. Results — p99 one-way latency (ns) at common load

| Size | kernel-udp | dpdk-off | dpdk-on (if run) |
|---|---|---|---|
| 64 | | | |
| 256 | | | |
| 1408 | | | |
| 16384 | | | |
| 1048576 | | | |

## 4. Results — offered throughput (msg/s) at common load

| Size | kernel-udp | dpdk-off | dpdk-on (if run) |
|---|---|---|---|
| 64 | | | |
| 256 | | | |
| 1408 | | | |
| 16384 | | | |
| 1048576 | | | |

## 5. Stress load (70% saturation) — p99 + offered

Fill the same two tables at the stress load; the gate verdict (§6) is computed
at common load only, but stress must not show loss/error growth.

| Size | kernel p99 (ns) | dpdk-off p99 (ns) | kernel offered | dpdk-off offered |
|---|---|---|---|---|
| 64 | | | | |
| 256 | | | | |
| 1408 | | | | |
| 16384 | | | | |
| 1048576 | | | | |

## 6. Gates (§12.3) — computed by `scripts/aws-dpdk-acceptance.sh` into `gates-verdict.txt`

**Primary gate (must PASS before any rollout):** for every size at common load,
`dpdk-off` must beat `kernel`:

- median p99 ≤ 90% of kernel p99 (≥10% lower), and
- offered throughput ≥ 99% of kernel offered.

Failure of either on any size → `FAIL` → no rollout. The acceptance script
reports `PRIMARY GATE: PASS|FAIL`.

**Saturation gate:** dpdk-off saturation must not be lower than kernel
saturation at any size (no throughput ceiling regression).

**Error/loss gate:** no unrecovered gaps (`gaps == 0`) and no backpressure
growth in any run; the `perf` receiver fails if a bad payload or zero messages
are seen.

## 7. ENA Express decision (dpdk-on)

From `ena-express-decision.json` (written by `decide_ena_express`): keep ENA
Express **ON only if** it improves median p99 by ≥5% at **every** size with no
>1% p50/throughput regression. Otherwise the decision is **OFF** and the cell
stays at `dpdk-off` (the shipped default). Rationale: SRD's benefits are
latency-focused at low load; an inconsistent or marginal win does not justify
the extra moving part.

Decision from the acceptance run: **OFF** (or ON with the size-by-size
numbers reproduced above).

## 8. Acceptance conclusion

| Check | Verdict |
|---|---|
| §11.3 functional matrix | PASS / FAIL |
| Primary gate (§12.3) | PASS / FAIL |
| ENA Express decision | OFF / ON |
| §13.2 rollback rehearsal | PASS / FAIL |

Bundle: `<link to artifacts bundle>`. Recorded against git SHA `<sha>`.
