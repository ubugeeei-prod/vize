"""Self-cost diff of two callgrind dumps.

usage: python3 cgdiff.py <base.callgrind.N> <head.callgrind.N> [top]
"""
import collections
import re
import sys


def load(path):
    names = {}
    self_cost = collections.Counter()
    meta = {}
    cur = None
    in_call = False
    for line in open(path, errors="replace"):
        line = line.rstrip("\n")
        if not line:
            continue
        if line.startswith(("desc:", "cmd:", "trigger:", "totals:", "summary:")):
            meta[line.split(":", 1)[0]] = line.split(":", 1)[1].strip()
            continue
        m = re.match(r"^(c?fn)=\((\d+)\)(?: (.*))?$", line)
        if m:
            kind, num, name = m.groups()
            if name is not None:
                names[num] = name
            if kind == "fn":
                cur = names.get(num, num)
            continue
        if line.startswith("calls="):
            in_call = True
            continue
        if line[0].isdigit() or line[0] in "+-*":
            parts = line.split()
            if len(parts) >= 2 and cur is not None:
                if in_call:
                    in_call = False
                else:
                    try:
                        self_cost[cur] += int(parts[1])
                    except ValueError:
                        pass
            continue
    return self_cost, meta


b, bm = load(sys.argv[1])
h, hm = load(sys.argv[2])
top = int(sys.argv[3]) if len(sys.argv) > 3 else 25
print("base", bm.get("trigger"), bm.get("totals") or bm.get("summary"))
print("head", hm.get("trigger"), hm.get("totals") or hm.get("summary"))
strip = lambda s: re.sub(r"::h[0-9a-f]{16}$", "", s)
bb, hh = collections.Counter(), collections.Counter()
for k, v in b.items():
    bb[strip(k)] += v
for k, v in h.items():
    hh[strip(k)] += v
diff = [(hh[k] - bb[k], k) for k in set(bb) | set(hh) if hh[k] != bb[k]]
diff.sort(key=lambda x: -abs(x[0]))
print("net", sum(d for d, _ in diff), "changed fns", len(diff))
for d, k in diff[:top]:
    print(f"{d:+8d} b={bb[k]:>9} h={hh[k]:>9} {k[:160]}")
