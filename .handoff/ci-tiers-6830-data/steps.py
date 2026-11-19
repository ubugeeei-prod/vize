import json, sys, datetime as dt, collections, re
P = lambda s: dt.datetime.fromisoformat(s.replace('Z', '+00:00')) if s else None
runs = json.load(open(sys.argv[1]))
pat = re.compile(sys.argv[2])
def pct(v, p):
    v = sorted(v)
    if not v: return float('nan')
    k = (len(v) - 1) * p; f = int(k)
    return v[f] + (v[min(f + 1, len(v) - 1)] - v[f]) * (k - f)
st = collections.defaultdict(list)
order = {}
for r in runs:
    for j in r['jobs']:
        if not pat.search(j['name']) or j['conclusion'] != 'success': continue
        for i, s in enumerate(j['steps']):
            if s['started'] and s['completed'] and s['conclusion'] != 'skipped':
                st[s['name']].append((P(s['completed']) - P(s['started'])).total_seconds())
                order.setdefault(s['name'], i)
for n in sorted(st, key=lambda n: order[n]):
    v = st[n]
    print(f"{len(v):4d} p50={pct(v,.5):6.1f}s p90={pct(v,.9):6.1f}s  {n[:110]}")
