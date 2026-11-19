"""Compare root-item partitions: usage cgu2.py a.txt b.txt"""
import sys, re, collections


def load(p):
    m = {}
    for line in open(p):
        if not line.startswith('MONO_ITEM fn '):
            continue
        item, _, rest = line.rstrip('\n').partition(' @@ ')
        name = item[len('MONO_ITEM fn '):]
        cg = re.findall(r'-cgu\.(\d+)\[(\w+)\]', rest)
        # local, non-generic-looking roots: crate-local path, single CGU
        if len(cg) != 1 or name.startswith('<') or '::<' in name or name.startswith('core::') or name.startswith('alloc::'):
            continue
        m[name] = int(cg[0][0])
    return m


a, b = load(sys.argv[1]), load(sys.argv[2])
common = set(a) & set(b)
table = collections.Counter((a[i], b[i]) for i in common)
ga = collections.defaultdict(set)
for (x, y), n in table.items():
    ga[x].add(y)
gb = collections.defaultdict(set)
for (x, y), n in table.items():
    gb[y].add(x)
split = {x: v for x, v in ga.items() if len(v) > 1}
merged = {y: v for y, v in gb.items() if len(v) > 1}
print(f'roots {len(common)} (only-a {len(set(a)-set(b))}, only-b {len(set(b)-set(a))}); '
      f'a-cgus split: {len(split)}, b-cgus mixing: {len(merged)}')
if '-v' in sys.argv:
    for x, v in sorted(split.items()):
        for y in sorted(v):
            items = sorted(i for i in common if a[i] == x and b[i] == y)
            print(f'  a{x}->b{y}: {len(items)} e.g. {items[:3]}')
