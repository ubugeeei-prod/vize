"""Self-cost per function from Callgrind dumps, and a diff.
usage: cgdiff.py <base.tgz> <head.tgz> <bench_id> [N]
Each tgz holds the run-1 dumps of one suite; the dump for <bench_id> is found by its trigger."""
import sys, tarfile, re, collections


def dumps(path):
    with tarfile.open(path) as t:
        for m in t.getmembers():
            if m.isfile():
                yield m.name, t.extractfile(m).read().decode('utf-8', 'replace')


def find(path, bench):
    for name, text in dumps(path):
        if f'desc: Trigger: Client Request: {bench}\n' in text:
            return text
    raise SystemExit(f'{bench} not found in {path}')


def selfcost(text):
    names = {}
    cost = collections.Counter()
    calls = collections.Counter()
    fn = None
    skip = 0
    cfn = None
    for line in text.splitlines():
        if not line:
            continue
        m = re.match(r'^(c?fn)=\((\d+)\)(?: (.*))?$', line)
        if m:
            kind, idx, nm = m.groups()
            if nm is not None:
                names[idx] = nm
            if kind == 'fn':
                fn = names.get(idx, idx)
            else:
                cfn = names.get(idx, idx)
            continue
        if line.startswith('calls='):
            skip = 1
            continue
        if line[0].isdigit() or line[0] in '+-*':
            parts = line.split()
            if len(parts) >= 2:
                v = int(parts[-1])
                if skip:
                    skip = 0
                    calls[(fn, cfn)] += v
                else:
                    cost[fn] += v
    return cost, calls


a, ac = selfcost(find(sys.argv[1], sys.argv[3]))
b, bc = selfcost(find(sys.argv[2], sys.argv[3]))
n = int(sys.argv[4]) if len(sys.argv) > 4 else 25
print('self total', sum(a.values()), '->', sum(b.values()))
for k in sorted(set(a) | set(b), key=lambda k: -abs(b[k] - a[k]))[:n]:
    if a[k] != b[k]:
        print(f'{b[k] - a[k]:+8d} {a[k]:9d} -> {b[k]:9d} {str(k)[:150]}')
print('--- inclusive call edges that moved')
for k in sorted(set(ac) | set(bc), key=lambda k: -abs(bc[k] - ac[k]))[:n]:
    if ac[k] != bc[k]:
        print(f'{bc[k] - ac[k]:+8d} {ac[k]:9d} -> {bc[k]:9d} {str(k[0])[:70]} -> {str(k[1])[:70]}')
