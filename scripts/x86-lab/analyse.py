#!/usr/bin/env python3
"""Summarises lab bench.csv files: per host and group, the median across reps of each
metric and the range across reps. Usage: analyse.py <bench.csv>..."""
import csv
import statistics
import sys
from collections import defaultdict


def fmt_ns(v):
    return f"{v / 1000:.2f}µs" if v >= 10_000 else f"{v:.0f}ns"


def summary(values):
    return statistics.median(values), min(values), max(values)


def main(paths):
    rtt = defaultdict(list)    # (host, group, label, mode) -> [(p50, p99, p999, p9999)]
    tput = defaultdict(list)   # (host, group, label) -> [median msgs/s]
    ps = defaultdict(list)     # (host, mode, n) -> [(ps_ns, plain_ns)]
    errors = []
    for path in paths:
        with open(path) as f:
            for row in csv.reader(f):
                host, group, layout, rep, kind = row[:5]
                if kind == "rtt":
                    label, mode = row[5], row[6]
                    p50, p90, p99, p999, p9999 = (float(x) for x in row[7:12])
                    samples = int(row[14]) if len(row) > 14 else 0
                    rtt[(host, group, label, mode)].append((p50, p99, p999, p9999, samples))
                elif kind == "tput":
                    tput[(host, group, row[5])].append(float(row[7]))
                elif kind == "pspoll":
                    mode, n = row[6], int(row[8])
                    ps[(host, mode, n)].append((float(row[9]), float(row[10])))
                else:
                    errors.append(row)

    hosts = sorted({k[0] for k in rtt} | {k[0] for k in tput} | {k[0] for k in ps})
    for host in hosts:
        print(f"\n## {host}")
        groups = sorted({k[1] for k in rtt if k[0] == host} | {k[1] for k in tput if k[0] == host})
        for group in groups:
            print(f"\n### {group}\n")
            print("| label | test | reps | p50 | p99 | p99.9 | p99.99 | msgs/s | fewest samples |")
            print("|---|---|---|---|---|---|---|---|---|")
            labels = sorted({k[2] for k in rtt if k[:2] == (host, group)} | {k[2] for k in tput if k[:2] == (host, group)})
            for label in labels:
                for mode in ("ipc", "udp"):
                    runs = rtt.get((host, group, label, mode))
                    if not runs:
                        continue
                    cells = []
                    for i in range(4):
                        m, lo, hi = summary([r[i] for r in runs])
                        cells.append(f"{fmt_ns(m)} ({fmt_ns(lo)}–{fmt_ns(hi)})")
                    fewest = min(r[4] for r in runs)
                    print(f"| {label} | {mode} rtt | {len(runs)} | " + " | ".join(cells) + f" | | {fewest} |")
                runs = tput.get((host, group, label))
                if runs:
                    m, lo, hi = summary(runs)
                    print(f"| {label} | ipc tput | {len(runs)} | | | | | {m / 1e6:.1f}M ({lo / 1e6:.1f}–{hi / 1e6:.1f}M) | |")
        modes = sorted({k[1] for k in ps if k[0] == host})
        if modes:
            print("\n### persistent subscription idle poll\n")
            print("| client | n | ns per persistent subscription poll | ns per subscription poll | ratio |")
            print("|---|---|---|---|---|")
            for mode in modes:
                for n in sorted({k[2] for k in ps if k[:2] == (host, mode)}):
                    runs = ps[(host, mode, n)]
                    p = statistics.median(r[0] for r in runs)
                    s = statistics.median(r[1] for r in runs)
                    print(f"| {mode} | {n} | {p:.1f} | {s:.1f} | {p / s:.1f}x |")
    if errors:
        print("\n## errors\n")
        for row in errors:
            print("- " + ",".join(row))


if __name__ == "__main__":
    main(sys.argv[1:])
