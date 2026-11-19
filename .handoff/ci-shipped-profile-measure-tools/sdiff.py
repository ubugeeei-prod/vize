"""usage: sdiff.py <dir-with-role-subdirs-or-files> base role1 role2 ... [--syms N]"""
import sys, os, json, gzip, collections, glob

d = sys.argv[1]
args = [a for a in sys.argv[2:] if not a.startswith('--')]
nsyms = 25
if '--syms' in sys.argv:
    nsyms = int(sys.argv[sys.argv.index('--syms') + 1])
    args = [a for a in args if a != str(nsyms)]
base, roles = args[0], args[1:]


def f(role, ext):
    c = glob.glob(os.path.join(d, '**', f'{role}.{ext}'), recursive=True)
    return c[0] if c else None


def syms(n):
    out = collections.Counter()
    for line in gzip.open(f(n, 'syms.gz'), 'rt'):
        p = line.rstrip('\n').split(' ', 2)
        if len(p) == 3 and p[1] in 'tTwW':
            out[p[2]] += int(p[0], 16)
    return out


bs = int(open(f(base, 'size')).read())
print("| role | ci-opt stripped bytes | vs base | release bytes | vs base |")
for r in [base] + roles:
    if not f(r, 'size'):
        print(f'| {r} | missing | |')
        continue
    s = int(open(f(r, 'size')).read())
    rel = ''
    if f(r, 'rsize') and f(base, 'rsize'):
        rb = int(open(f(base, 'rsize')).read())
        rs = int(open(f(r, 'rsize')).read())
        rel = f' {rs:,} | {rs - rb:+,} ({100.0 * (rs - rb) / rb:+.4f}%) |'
    print(f'| {r} | {s:,} | {s - bs:+,} ({100.0 * (s - bs) / bs:+.4f}%) |{rel}')
for kind in ('ic', 'fic', 'sic'):
    if not f(base, f'{kind}.json'):
        continue
    ba = json.load(open(f(base, f'{kind}.json')))
    for r in roles:
        p = f(r, f'{kind}.json')
        if not p:
            print(f'{kind} {r}: missing')
            continue
        hb = json.load(open(p))
        moved = [(k, ba[k]['instructions'], hb.get(k, {}).get('instructions')) for k in sorted(ba)
                 if ba[k]['instructions'] != hb.get(k, {}).get('instructions')]
        up = [m for m in moved if m[2] is None or m[2] > m[1]]
        print(f'\n=== {kind} {base} -> {r}: {len(moved)} moved, {len(up)} up, {len(ba) - len(moved)} identical')
        for k, x, y in moved:
            dd = (y - x) if y is not None else 0
            print(f'| {k} | {x:,} | {y:,} | {dd:+,} | {100.0 * dd / x:+.3f}% |')
if nsyms:
    a = syms(base)
    for r in roles:
        if not f(r, 'syms.gz'):
            continue
        b = syms(r)
        print(f'\n=== {base} -> {r}: total text {sum(a.values())} -> {sum(b.values())} ({sum(b.values()) - sum(a.values()):+d})')
        for k in sorted(set(a) | set(b), key=lambda k: -abs(b[k] - a[k]))[:nsyms]:
            if a[k] != b[k]:
                print(f'{b[k]-a[k]:+7d} {a[k]:7d} -> {b[k]:7d} {k[:150]}')
