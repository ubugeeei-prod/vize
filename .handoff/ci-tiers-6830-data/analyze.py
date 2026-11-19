import json, sys, datetime as dt, collections, re
P = lambda s: dt.datetime.fromisoformat(s.replace('Z', '+00:00')) if s else None
runs = json.load(open(sys.argv[1]))

def norm(name):
    name = re.sub(r'Affected tooling \(.*\)', 'Affected tooling (shard)', name)
    return name

def pct(v, p):
    v = sorted(v)
    if not v: return float('nan')
    k = (len(v) - 1) * p; f = int(k)
    return v[f] + (v[min(f + 1, len(v) - 1)] - v[f]) * (k - f)

durs = []
last = collections.Counter()
stats = collections.defaultdict(lambda: {'run': [], 'delay': [], 'end': [], 'queue': []})
for r in runs:
    c = P(r['created'])
    jobs = [j for j in r['jobs'] if j['conclusion'] not in ('skipped', None) and j['started']]
    end = max(P(j['completed']) for j in jobs)
    durs.append((end - c).total_seconds() / 60)
    # the last non-report job
    work = [j for j in jobs if not re.search(r'report$|^test-report$', j['name'])]
    lj = max(work, key=lambda j: P(j['completed']))
    last[norm(lj['name'])] += 1
    for j in jobs:
        n = norm(j['name'])
        s = stats[n]
        s['run'].append((P(j['completed']) - P(j['started'])).total_seconds() / 60)
        s['delay'].append((P(j['started']) - c).total_seconds() / 60)
        s['end'].append((P(j['completed']) - c).total_seconds() / 60)
        s['queue'].append((P(j['started']) - P(j['created'])).total_seconds() / 60)
print(f"runs={len(runs)} p50={pct(durs,.5):.1f} p90={pct(durs,.9):.1f} min={min(durs):.1f}")
print("last finishing work job:", last.most_common(10))
print(f"{'job':70s} {'n':>3s} {'run50':>6s} {'run90':>6s} {'start50':>7s} {'q50':>5s} {'end50':>6s} {'end90':>6s}")
for n, s in sorted(stats.items(), key=lambda kv: -pct(kv[1]['end'], .5)):
    print(f"{n[:70]:70s} {len(s['run']):3d} {pct(s['run'],.5):6.1f} {pct(s['run'],.9):6.1f} {pct(s['delay'],.5):7.1f} {pct(s['queue'],.5):5.1f} {pct(s['end'],.5):6.1f} {pct(s['end'],.9):6.1f}")
