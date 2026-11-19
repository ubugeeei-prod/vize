"""usage: offdiff.py <base-dir> <head-dir>   (official level-instruction-counts artifacts)
Prints a markdown table of every moved row (all three kinds, run-1..3 must agree)."""
import sys, json, glob, os


def load(d, prefix):
    c = glob.glob(os.path.join(d, f'{prefix}-*', 'measurement.json'))
    if not c:
        return None
    m = json.load(open(c[0]))
    rows = {}
    for run in m['runs']:
        for k, v in run.items():
            rows.setdefault(k, set()).add(v['instructions'])
    return rows


total = 0
moved = []
for prefix in ('instruction-counts', 'formatter-instruction-counts', 'sfc-instructions'):
    a, b = load(sys.argv[1], prefix), load(sys.argv[2], prefix)
    if a is None or b is None:
        print(f'{prefix}: missing')
        continue
    for k in sorted(a):
        total += 1
        x, y = a[k], b.get(k, set())
        if len(x) != 1 or len(y) != 1:
            print(f'NOTE {k}: runs disagree {sorted(x)} {sorted(y)}')
        x0, y0 = min(x), min(y) if y else None
        if x0 != y0:
            moved.append((k, x0, y0))
print('| Row | Base | Head | Δ |')
print('| --- | ---: | ---: | ---: |')
for k, x, y in moved:
    d = y - x
    print(f'| `{k}` | {x:,} | {y:,} | {d:+,} ({100.0 * d / x:+.3f}%) |')
up = [m for m in moved if m[2] > m[1]]
print(f'| other {total - len(moved)} rows | | | 0 (identical) |')
print(f'\nmoved {len(moved)}, up {len(up)}, total {total}')
