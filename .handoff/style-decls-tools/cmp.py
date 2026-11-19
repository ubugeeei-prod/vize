"""usage: cmp.py <a> <b>: compare lint outputs of two roles; for differing files, count findings per rule."""
import sys, os, json, hashlib, collections
L = '/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb/style/lint'
a, b = sys.argv[1], sys.argv[2]


def findings(p):
    data = json.load(open(p))
    out = collections.Counter()
    items = []

    def walk(x):
        if isinstance(x, dict):
            rule = x.get('rule') or x.get('ruleId') or x.get('code')
            if rule and ('start' in x or 'line' in x or 'span' in x or 'offset' in x or 'range' in x):
                out[rule] += 1
                items.append(json.dumps(x, sort_keys=True))
            for v in x.values():
                walk(v)
        elif isinstance(x, list):
            for v in x:
                walk(v)
    walk(data)
    return out, collections.Counter(items)


for f in sorted(os.listdir(f'{L}/{b}')):
    if not f.endswith('.json'):
        continue
    pa, pb = f'{L}/{a}/{f}', f'{L}/{b}/{f}'
    if not os.path.exists(pa):
        print(f'{f}: missing in {a}')
        continue
    ha = hashlib.sha256(open(pa, 'rb').read()).hexdigest()[:12]
    hb = hashlib.sha256(open(pb, 'rb').read()).hexdigest()[:12]
    if ha == hb:
        print(f'{f}: identical ({ha})')
        continue
    ca, ia = findings(pa)
    cb, ib = findings(pb)
    added = ib - ia
    removed = ia - ib
    per = collections.Counter()
    for k, n in added.items():
        per[json.loads(k).get('rule') or json.loads(k).get('ruleId') or json.loads(k).get('code'), '+'] += n
    for k, n in removed.items():
        per[json.loads(k).get('rule') or json.loads(k).get('ruleId') or json.loads(k).get('code'), '-'] += n
    print(f'{f}: differs {ha} -> {hb}; findings {sum(ca.values())} -> {sum(cb.values())}; ' + ', '.join(f'{r} {s}{n}' for (r, s), n in sorted(per.items())))
