#!/usr/bin/env python3
"""Per-function self-cost diff of one benchmark row between two instruction-count runs.

usage: lbcg.py <base-run-id> <head-run-id> <bench-id> [top]
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
            head = handle.read(4000)
        if f'Client Request: {bench}\n' in head:
            return path
    sys.exit(f'no dump for {bench} in {run}')


def self_costs(path):
    names = {}
    costs = defaultdict(int)
    current = None
    skip_next = False
    for line in open(path, errors='replace'):
        line = line.rstrip('\n')
        if skip_next:
            skip_next = False
            continue
        if line.startswith('fn='):
            m = re.match(r'fn=\((\d+)\)(?: (.*))?', line)
            if m:
                ident, name = m.groups()
                if name:
                    names[ident] = name
                current = names.get(ident, ident)
            else:
                current = line[3:]
            continue
        if line.startswith('calls='):
            skip_next = True
            continue
        if line.startswith(('cfn=', 'cfi=', 'cfl=', 'cob=')):
            m = re.match(r'cfn=\((\d+)\) (.*)', line)
            if m:
                names[m.group(1)] = m.group(2)
            continue
        if current and line and (line[0].isdigit() or line[0] in '+-*'):
            parts = line.split()
            if len(parts) >= 2 and parts[-1].isdigit():
                costs[current] += int(parts[-1])
    return costs


def main():
    base_run, head_run, bench = sys.argv[1:4]
    top = int(sys.argv[4]) if len(sys.argv) > 4 else 25
    base = self_costs(find(base_run, bench))
    head = self_costs(find(head_run, bench))
    print(f'{bench}: base {sum(base.values()):,} head {sum(head.values()):,}')
    rows = [(head.get(k, 0) - base.get(k, 0), k) for k in set(base) | set(head)]
    rows = [r for r in rows if r[0]]
    rows.sort(key=lambda r: -abs(r[0]))
    for delta, name in rows[:top]:
        print(f'{delta:+8d}  base {base.get(name, 0):>9,}  head {head.get(name, 0):>9,}  {name[:150]}')


main()
