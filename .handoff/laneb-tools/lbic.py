#!/usr/bin/env python3
"""Compare instruction-count artifacts: lbic.py <base-run> <head-run> <base-twin> <head-twin> [kind]
kind: instruction-counts (default), formatter-instruction-counts, sfc-instructions."""
import json, sys, glob, statistics, os
IC = '/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/laneb-ic'
kind = sys.argv[5] if len(sys.argv) > 5 else 'instruction-counts'
def load(run):
    paths = [p for p in glob.glob(f'{IC}/{run}/{kind}-*/measurement.json')]
    if not paths: sys.exit(f'missing {kind} for {run}')
    d = json.load(open(paths[0]))
    rows = {}
    for r in d['runs']:
        for k, v in r.items():
            rows.setdefault(k, []).append(v['instructions'])
    return {k: statistics.median(v) for k, v in rows.items()}, {k: len(set(v)) == 1 for k, v in rows.items()}
b, bs = load(sys.argv[1]); h, hs = load(sys.argv[2]); b2, _ = load(sys.argv[3]); h2, _ = load(sys.argv[4])
aa_b = sum(1 for k in b if b2.get(k) != b[k]); aa_h = sum(1 for k in h if h2.get(k) != h[k])
print(f'rows base={len(b)} head={len(h)}; A/A base differs on {aa_b} rows, head differs on {aa_h} rows')
moved = []
for k in sorted(set(b) | set(h)):
    if b.get(k) != h.get(k):
        bb, hh = b.get(k), h.get(k)
        pct = (hh - bb) / bb * 100 if bb and hh else float('nan')
        moved.append((pct, k, bb, hh))
moved.sort()
print('| Row | Base | Head | Δ |'); print('| --- | ---: | ---: | ---: |')
for pct, k, bb, hh in moved:
    print(f'| `{k}` | {int(bb):,} | {int(hh):,} | {pct:+.4f}% |')
print(f'other {len(b) - len(moved)} rows identical; regressions: {sum(1 for m in moved if m[0] > 0)}')
