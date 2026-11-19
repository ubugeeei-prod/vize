"""Simulate rustc's merge_codegen_units from an unmerged partition.

usage: sim.py <name> [actual16-mono.txt] [--n 16] [--adjust cguN:+delta ...]
reads rbs/<name>-init.txt and rbs/stats-<name>/<crate>.mono_items.json
"""
import sys, re, json, collections, glob, os

S = '/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb/rbs'


def skeleton(name):
    out = []
    i = 0
    n = len(name)
    while i < n:
        c = name[i]
        if c == '<':
            prev = ''.join(out).rstrip()
            is_args = bool(prev) and (prev[-1].isalnum() or prev[-1] in '_:>}' ) and not prev.endswith('::') or prev.endswith('::') and False
            # generic args follow an identifier, `>`, or a turbofish `::`
            if prev.endswith('::'):
                is_args = True
                # drop the turbofish `::`
                while out and out[-1] == ':':
                    out.pop()
            if is_args:
                depth = 0
                while i < n:
                    if name[i] == '<':
                        depth += 1
                    elif name[i] == '>':
                        depth -= 1
                        if depth == 0:
                            i += 1
                            break
                    elif name[i] == '-' and i + 1 < n and name[i + 1] == '>':
                        i += 2
                        continue
                    i += 1
                continue
        if c == '{':
            # closures with spans
            j = name.find('}', i)
            tok = name[i:j + 1]
            tok = re.sub(r'\{closure@[^}]*\}', '{closure}', tok)
            out.append(tok)
            i = j + 1
            continue
        out.append(c)
        i += 1
    s = ''.join(out)
    s = s.replace("'_", '')
    return s


def load_init(path):
    cgus = collections.defaultdict(set)
    count = collections.Counter()
    for line in open(path):
        if not line.startswith('MONO_ITEM '):
            continue
        item, _, rest = line.rstrip('\n').partition(' @@ ')
        item = item[10:]
        for tok in rest.split():
            mm = re.match(r'.*-cgu\.(\d+)\[(\w+)\]', tok)
            if mm:
                cgus[int(mm.group(1))].add(item)
                count[item] += 1
    return cgus, count


def load_sizes(dirpath):
    f = glob.glob(os.path.join(dirpath, '*.mono_items.json'))[0]
    sizes = {}
    for e in json.load(open(f)):
        sizes[skeleton(e['name'])] = e['size_estimate']
    return sizes


def item_size(item, sizes, missing):
    if item.startswith('static '):
        return 1
    if item.startswith('global_asm'):
        return 1
    name = item[3:] if item.startswith('fn ') else item
    sk = skeleton(name)
    if sk in sizes:
        return sizes[sk]
    missing[sk] += 1
    return 0


def simulate(name, n=16, adjust=None, verbose=False):
    cgus, count = load_init(f'{S}/{name}-init.txt')
    sizes = load_sizes(f'{S}/stats-{name}')
    missing = collections.Counter()
    isz = {}
    for c, items in cgus.items():
        for it in items:
            if it not in isz:
                isz[it] = item_size(it, sizes, missing)
    if adjust:
        for it, d in adjust.items():
            isz[it] = isz.get(it, 0) + d
    inlined = {it for it, k in count.items() if k > 1}
    # units: list of [name(int initial id), items dict, size]
    units = []
    for c in sorted(cgus):
        items = set(cgus[c])
        units.append([c, items, sum(isz[i] for i in items), [c]])
    while len(units) > n:
        units.sort(key=lambda u: -u[2])  # stable
        dst = units[n - 1]
        max_overlap = 0
        max_i = n
        for i in range(n, len(units)):
            src = units[i]
            if src[2] <= max_overlap:
                break
            a, b = (dst[1], src[1]) if len(dst[1]) <= len(src[1]) else (src[1], dst[1])
            ov = sum(isz[it] for it in a if it in inlined and it in b)
            if ov > max_overlap:
                max_overlap = ov
                max_i = i
        src = units[max_i]
        units[max_i] = units[-1]
        units.pop()
        dst = units[n - 1]
        dst[1] |= src[1]
        dst[2] = sum(isz[i] for i in dst[1])
        dst[3] = dst[3] + src[3]
        if verbose:
            print('merge', src[3], 'into', dst[0], 'ov', max_overlap, 'size', dst[2])
    if missing and verbose:
        print('missing sizes', len(missing), missing.most_common(10))
    return units, cgus


def roots_signature(units, cgus, count):
    sig = []
    for u in units:
        roots = frozenset(it for it in u[1] if count[it] == 1)
        sig.append(roots)
    return sig


def actual_groups(path):
    g = collections.defaultdict(set)
    cnt = collections.Counter()
    rows = []
    for line in open(path):
        if not line.startswith('MONO_ITEM '):
            continue
        item, _, rest = line.rstrip('\n').partition(' @@ ')
        cg = re.findall(r'-cgu\.(\d+)\[', rest)
        rows.append((item[10:], cg))
    for it, cg in rows:
        if len(cg) == 1:
            g[int(cg[0])].add(it)
    return g


def compare(units, count, path):
    actual = {}
    for line in open(path):
        if not line.startswith('MONO_ITEM '):
            continue
        item, _, rest = line.rstrip('\n').partition(' @@ ')
        cg = re.findall(r'-cgu\.(\d+)\[', rest)
        if len(cg) == 1:
            actual[item[10:]] = int(cg[0])
    simmap = {}
    for k, u in enumerate(units):
        for it in u[1]:
            if count[it] == 1:
                simmap[it] = k
    pairs = collections.defaultdict(set)
    for it, k in simmap.items():
        if it in actual:
            pairs[k].add(actual[it])
    bad = {k: v for k, v in pairs.items() if len(v) > 1}
    inv = collections.defaultdict(set)
    for k, v in pairs.items():
        for a in v:
            inv[a].add(k)
    bad2 = {a: v for a, v in inv.items() if len(v) > 1}
    return f'sim units spanning >1 actual cgu: {len(bad)}; actual cgus spanning >1 sim unit: {len(bad2)}'


if __name__ == '__main__':
    name = sys.argv[1]
    units, cgus = simulate(name, verbose='-v' in sys.argv)
    _, count = load_init(f'{S}/{name}-init.txt')
    if len(sys.argv) > 2 and not sys.argv[2].startswith('-'):
        print(compare(units, count, sys.argv[2]))
    print('sizes', sorted((u[2] for u in units), reverse=True))
