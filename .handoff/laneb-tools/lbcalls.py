#!/usr/bin/env python3
"""Callees of one function (call counts and inclusive cost) in base vs head dumps of one row.

usage: lbcalls.py <base-run> <head-run> <bench-id> <function-substring>
"""
import glob
import re
import sys
from collections import defaultdict

IC = '/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/laneb-ic'


def find(run, bench):
    for path in glob.glob(f'{IC}/{run}/instruction-counts-*/run-1/*.callgrind*'):
        if path.endswith(('.stderr', '.stdout')):
            continue
        with open(path, errors='replace') as handle:
            if f'Client Request: {bench}\n' in handle.read(4000):
                return path
    sys.exit(f'no dump for {bench} in {run}')


def callees(path, needle):
    names = {}
    out = defaultdict(lambda: [0, 0])
    current = None
    callee = None
    pending = None
    for line in open(path, errors='replace'):
        line = line.rstrip('\n')
        m = re.match(r'(c?fn)=\((\d+)\)(?: (.*))?', line)
        if m:
            kind, ident, name = m.groups()
            if name:
                names[ident] = name
            if kind == 'fn':
                current = names.get(ident, ident)
            else:
                callee = names.get(ident, ident)
            continue
        if line.startswith('calls='):
            pending = int(line.split('=')[1].split()[0])
            continue
        if pending is not None:
            parts = line.split()
            if current and needle in current and parts and parts[-1].isdigit():
                out[callee][0] += pending
                out[callee][1] += int(parts[-1])
            pending = None
    return out


base_run, head_run, bench, needle = sys.argv[1:5]
b = callees(find(base_run, bench), needle)
h = callees(find(head_run, bench), needle)
rows = []
for k in set(b) | set(h):
    bc, bi = b.get(k, [0, 0])
    hc, hi = h.get(k, [0, 0])
    if (bc, bi) != (hc, hi):
        rows.append((hi - bi, bc, hc, bi, hi, k))
rows.sort(key=lambda r: -abs(r[0]))
for d, bc, hc, bi, hi, k in rows[:25]:
    print(f'{d:+8d} calls {bc}->{hc} incl {bi:,}->{hi:,}  {k[:140]}')
print(len(rows), 'callees differ')
