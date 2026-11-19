import sys, re, collections


def load(p):
    m = collections.defaultdict(set)  # item -> set of (cgu, linkage)
    for line in open(p):
        if not line.startswith('MONO_ITEM '):
            continue
        item, _, rest = line.rstrip('\n').partition(' @@ ')
        for tok in rest.split():
            mm = re.match(r'.*-cgu\.(\d+)\[(\w+)\]', tok)
            if mm:
                m[item[10:]].add((int(mm.group(1)), mm.group(2)))
    return m


a, b = load(sys.argv[1]), load(sys.argv[2])


def groups(m):
    g = collections.defaultdict(set)
    for it, s in m.items():
        if len(s) == 1:
            (c, l), = s
            g[c].add(it)
    return g


def modname(it):
    x = re.sub(r'^(fn|static) ', '', it)
    x = x.lstrip('<')
    return '::'.join(x.split('::')[:3])


def sig(g):
    return {c: frozenset(modname(it) for it in items) for c, items in g.items()}


ga, gb = groups(a), groups(b)
sa, sb = sig(ga), sig(gb)
print('cgus', len(sa), len(sb))
same = set(sa.values()) & set(sb.values())
print('identical module groups', len(same))
for c, s in sorted(sa.items()):
    if s not in same:
        print('BASE', c, sorted(s))
for c, s in sorted(sb.items()):
    if s not in same:
        print('HEAD', c, sorted(s))
n = 0
for it in sorted(set(a) | set(b)):
    la = sorted(l for _, l in a.get(it, ()))
    lb = sorted(l for _, l in b.get(it, ()))
    if la != lb and n < 80:
        n += 1
        print('LINK', la, lb, it[:200])
# inline-item copy counts per CGU-group
