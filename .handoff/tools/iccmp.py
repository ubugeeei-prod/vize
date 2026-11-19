"""Compare instruction-count artifacts: base, head, base twin, head twin.

usage: python3 iccmp.py <base_dir> <head_dir> <base_twin_dir> <head_twin_dir>
"""
import glob
import json
import os
import sys


def rows(d):
    out = {}
    for m in glob.glob(os.path.join(d, "**", "measurement.json"), recursive=True):
        runs = json.load(open(m))["runs"]
        for name in runs[0]:
            vals = {r[name]["instructions"] for r in runs}
            assert len(vals) == 1, (m, name, vals)
            out[name] = vals.pop()
    return out


b, h, bt, ht = (rows(x) for x in sys.argv[1:5])
names = sorted(set(b) | set(h))
aa_b = sum(1 for n in names if b.get(n) != bt.get(n))
aa_h = sum(1 for n in names if h.get(n) != ht.get(n))
print(f"rows: {len(names)}; A/A base diffs: {aa_b}; A/A head diffs: {aa_h}")
moved = [(n, b.get(n), h.get(n)) for n in names if b.get(n) != h.get(n)]
print(f"moved rows: {len(moved)}")
for n, x, y in moved:
    pct = (y - x) / x * 100 if x and y else float("nan")
    print(f"| `{n}` | {x:,} | {y:,} | {pct:+.4f}% |")
