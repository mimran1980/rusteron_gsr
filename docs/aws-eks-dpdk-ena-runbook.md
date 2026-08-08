# AWS EKS DPDK ENA — rollout and rollback runbook (plan §13)

Operational guide for rolling the `rusteron-media-driver` DPDK ENA
kernel-bypass transport out to production EKS cells and — when the acceptance
gates say so — back to the default kernel-UDP transport, with no binary
change. Covers §12 benchmarking prerequisites and the §13.2 rollback
rehearsal that the acceptance script automates.

Scope: Amazon Linux 2023, Linux x86_64 Nitro, one Media Driver pod per
dedicated node with its own ENA pair. The node's **primary ENA is never
touched** — it stays kernel-owned and carries EKS networking, DNS, telemetry,
and the kernel-UDP rollback path.

---

## 1. Prerequisites

- Cluster cell per §10 with `deploy/aws-eks/` applied
  (`kubectl apply -k deploy/aws-eks/`): device plugin, Media Driver DaemonSet,
  node bootstrap systemd unit on every dedicated node.
- Node label `rusteron-dpdk/ena=true` on the dedicated nodes and the
  `rusteron-dpdk` taint tolerated by the DaemonSet.
- Built binaries (see §4 of the acceptance script header):
  - `dpdk-harness` with `--features dpdk` (release),
  - `media_driver` release binary,
  - and a machine with `aws`, `kubectl`, `ssh`/`scp`, `jq`.
- **The Media Driver DaemonSet scaled to 0 in the test cell** — the acceptance
  harness binds the ENA pairs itself, so it must not race the DaemonSet:
  ```bash
  kubectl -n <cell-ns> scale deploy --replicas=0 rusteron-media-driver 2>/dev/null || \
  kubectl -n <cell-ns> scale ds rusteron-media-driver --replicas=0
  ```
  (For a DaemonSet the field is `ds`, not `deploy`.)

## 2. Acceptance run (§11.3 + §12)

```bash
# From the repo root, after `cargo build -p dpdk-harness --features dpdk --release`
# and `cargo build -p rusteron-media-driver --bin media_driver --release`:
scripts/aws-dpdk-acceptance.sh all
```

Phases are rerunnable (`functional | bench | rollback | metadata`) and all
artifacts land in `$RUSTERON_ACCEPTANCE_ARTIFACTS/run-<timestamp>/` — a
versioned bundle with the per-scenario reports, raw latency samples, gate
verdict, and `env-metadata.json` (§12.2). See
`docs/aws-eks-dpdk-benchmark-report.md` for how to turn that bundle into the
benchmark report.

Key environment overrides (all optional):

| Variable | Default | Purpose |
|---|---|---|
| `RUSTERON_ACCEPTANCE_CELL` | `dpdk-test` | cell name for metadata |
| `RUSTERON_ACCEPTANCE_ARTIFACTS` | `/tmp/rusteron-aws-acceptance` | artifact root |
| `RUSTERON_HARNESS_BIN` | `target/release/dpdk-harness` | prebuilt harness |
| `RUSTERON_MEDIA_DRIVER_BIN` | `target/release/media_driver` | prebuilt driver |
| `RUSTERON_ACCEPTANCE_SSH_USER` | `ec2-user` | node SSH user |
| `RUSTERON_ACCEPTANCE_SSH_KEY` | (none) | optional `-i` key |
| `RUSTERON_ACCEPTANCE_WARMUP` | 30 | saturation warm-up (s) |
| `RUSTERON_ACCEPTANCE_RUNS` | 10 | runs per size/load |
| `RUSTERON_ACCEPTANCE_RUN_SECS` | 60 | run duration (s) |
| `RUSTERON_ACCEPTANCE_SIZES` | `64 256 1408 16384 1048576` | message sizes |
| `RUSTERON_ACCEPTANCE_COMMON_FRAC` | 0.5 | common-load fraction of saturation |
| `RUSTERON_ACCEPTANCE_STRESS_FRAC` | 0.7 | stress-load fraction |

The script picks the first two `rusteron-dpdk/ena=true` nodes, so the cell must
have at least two ready nodes.

## 3. Rollout (§13.1)

Sequence — non-prod first, then one prod AZ cell at a time:

1. **Non-prod cell.** Run the full acceptance bundle (§2). The bundle must end
   with `PRIMARY GATE: PASS` (see `gates-verdict.txt`) before anything ships.
   The ENA Express decision lands in `ena-express-decision.json`; **default:
   ENA Express stays OFF** unless the evidence says otherwise (see §12.3 of the
   benchmark report).
2. **One prod AZ cell.** Apply `deploy/aws-eks/` with the DaemonSet scheduling
   on that cell's dedicated nodes. Confirm per-pod health before advancing:
   ```bash
   kubectl -n <ns> get pods -o wide -l app=rusteron-media-driver
   kubectl -n <ns> logs -l app=rusteron-media-driver | grep -E 'transport backend|media driver started'
   ```
   Every pod must log `transport backend: dpdk-ena` and `media driver started`.
   Watch `rusteron-media-driver`-reported Aeron counters and the pod's
   `backpressure` / `gaps` (harness reports) for the first few hours.
3. **Advance one AZ cell at a time.** Leave a rollback window (e.g. two
   release cycles) between cells. There is no "flagship" toggle — the transport
   is selected per deployment by `RUSTERON_MEDIA_DRIVER_TRANSPORT`, so cells
   can hold different transports simultaneously. This is deliberate: a cell is
   the smallest revertible unit.

Never change the transport on more than one AZ cell in a single change. A
single failure domain failing is the intended blast radius.

## 4. Rollback (§13.2)

Rollback is a **config change, not a binary change**: the same image supports
both transports, selected at driver start from the environment.

```bash
# 1. Flip the deployment's transport env to the default (kernel UDP).
kubectl -n <ns> set env deploy/rusteron-media-driver \
  RUSTERON_MEDIA_DRIVER_TRANSPORT=default
# 2. The DaemonSet rolls one pod at a time (strategy default: OnDelete for
#    DaemonSets — patch to RollingUpdate first if you want automated rollout).
# 3. Clients reconnect to the *primary-ENA* endpoints. The DPDK harness's UDP
#    env is exactly this: MDC control + endpoints are the node private IPs.
```

What must be true after rollback:

- Every pod logs `transport backend: socket` (not `dpdk-ena`).
- Clients resume publishing/receiving over the nodes' private-IP endpoints
  (primary ENA, kernel UDP) — **no client change**.
- The secondary ENAs stay bound to `vfio-pci` on the host (isolated), so the
  DPDK path remains ready to re-enable. Rollback does **not** unbind them:
  ```bash
  for node in $(kubectl get nodes -l rusteron-dpdk/ena=true -o name); do
    pci=$(ssh ec2-user@$node "jq -r '.pairs[0].sender.pci' /var/lib/rusteron-dpdk/ena-pairs.json")
    ssh ec2-user@$node "readlink /sys/bus/pci/devices/$pci/driver | xargs basename"  # must print vfio-pci
  done
  ```
- The kernel-UDP path carries the workload at the §12.3 kernel baseline —
  which the acceptance run measured, so rollback performance is a known
  quantity, not a surprise.

The acceptance script rehearses exactly this (§`phase_rollback`): it reruns a
bidirectional scenario with `RUSTERON_MEDIA_DRIVER_TRANSPORT` unset (default)
over the primary ENA and asserts both nodes still hold `vfio-pci` on their
secondaries afterwards.

**To roll forward again** after a rollback: restore
`RUSTERON_MEDIA_DRIVER_TRANSPORT=dpdk-ena` and confirm the backend log lines,
then watch the same counters as §3.

## 5. Operational notes

- **ENA Express** is toggled per-ENI with `scripts/toggle-ena-express.sh`
  (`on|off`). It requires SRD-capable instance types and an ENA that supports
  it; the script verifies the resulting `EnaSrdSpecification`. The acceptance
  gate keeps it OFF unless it delivers a ≥5% median-p99 improvement at every
  message size with no throughput regression.
- **The §12.1 kernel baseline temporarily returns the secondaries to the
  kernel `ena` driver**, which the acceptance script does and then restores to
  `vfio-pci` (same driver_override + probe path the bootstrap uses). If the
  acceptance run fails during the kernel baseline, restore the pair by
  re-running the node bootstrap (it is idempotent). The primary ENA remains
  the §13.2 rollback path and is never touched.
- **Fragments**: the DPDK path never fragments (DF + MTU enforced in the
  DPDK transmit path, unit-tested). The kernel-UDP side is checked with
  tcpdump during acceptance; any `frag` capture fails the run.
- **A failed DPDK start is a loud failure.** `configure_media_transport_from_env`
  returns an error and the process exits nonzero — it never silently falls back
  to the socket driver. Treat a `transport backend: socket` line as an explicit
  operator decision, not a default.
