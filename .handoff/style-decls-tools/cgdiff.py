"""usage: cgdiff.py <dir> <base> <role> <lane-file-suffix> [n]"""
import re, sys, collections
d, base, role, lane = sys.argv[1:5]
n = int(sys.argv[5]) if len(sys.argv) > 5 else 40


def load(p):
    out = collections.Counter()
    for line in open(p, errors='replace'):
        m = re.match(r'\s*([\d,]+) \(\s*[\d.]+%\)\s+(.*)$', line)
        if not m or 'PROGRAM TOTALS' in line:
            continue
        fn = m.group(2)
        fn = re.sub(r' \[[^\]]*\]$', '', fn)
        fn = re.sub(r'^[^:]*:', '', fn, count=1)
        out[fn] += int(m.group(1).replace(',', ''))
    return out


a = load(f'{d}/{base}-{lane}.txt')
b = load(f'{d}/{role}-{lane}.txt')
print(f'{lane}: {base} {sum(a.values()):,} -> {role} {sum(b.values()):,} ({sum(b.values()) - sum(a.values()):+,})')
for k in sorted(set(a) | set(b), key=lambda k: -abs(b[k] - a[k]))[:n]:
    if a[k] != b[k]:
        print(f'{b[k]-a[k]:+12,} {a[k]:13,} {k[:170]}')
