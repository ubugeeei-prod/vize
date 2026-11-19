import json, glob, collections, sys
d = sys.argv[1]
f = sorted(glob.glob(d + '/*.cpuprofile'))[-1]
p = json.load(open(f))
nodes = {n['id']: n for n in p['nodes']}
parent = {}
for n in p['nodes']:
    for c in n.get('children', []):
        parent[c] = n['id']
self_t = collections.Counter(); incl = collections.Counter()
for s, dt in zip(p['samples'], p['timeDeltas']):
    n = nodes[s]; cf = n['callFrame']
    self_t[(cf['functionName'] or '(anon)') + ' ' + cf['url'].split('/')[-1] + ':' + str(cf['lineNumber'])] += dt
    seen = set(); cur = s
    while cur in nodes:
        cf = nodes[cur]['callFrame']
        k = (cf['functionName'] or '(anon)') + ' ' + cf['url'].split('/')[-1] + ':' + str(cf['lineNumber'])
        if k not in seen: incl[k] += dt; seen.add(k)
        cur = parent.get(cur)
        if cur is None: break
tot = sum(self_t.values())
print('total', tot / 1000, 'ms')
for k, v in self_t.most_common(15): print(f"self {v/1000:8.0f}ms {100*v/tot:5.1f}% {k}")
for k, v in incl.most_common(25): print(f"incl {v/1000:8.0f}ms {100*v/tot:5.1f}% {k}")
