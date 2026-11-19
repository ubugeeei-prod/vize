"""Compare instruction rows across downloaded level-instruction-counts runs.

usage: ic_compare.py <dir-a> <dir-b> [<dir-c> ...]
Each dir holds the three artifacts of one run. Prints moved rows and counts.
"""
import glob
import re
import sys

ROW = re.compile(r'^([A-Za-z0-9_.\-]+) = \{.*?instructions = (\d+)', re.M)


def rows(directory):
    out = {}
    for path in sorted(glob.glob(directory + "/*/measured-budgets.toml")):
        kind = path.split("/")[-2].rsplit("-", 2)[0]
        kind = re.sub(r"-[0-9a-f]{40}$", "", kind)
        for name, value in ROW.findall(open(path).read()):
            out[(kind, name)] = int(value)
    return out


runs = [rows(d) for d in sys.argv[1:]]
keys = sorted(set().union(*runs))
moved = 0
for key in keys:
    values = [run.get(key) for run in runs]
    if len(set(values)) > 1:
        moved += 1
        base = values[0]
        deltas = " ".join(
            f"{v}" + (f" ({(v - base) / base * 100:+.4f}%)" if base and v is not None else "")
            for v in values
        )
        print(f"{key[0]}/{key[1]}: {deltas}")
print(f"rows: {len(keys)}, moved: {moved}, identical: {len(keys) - moved}")
