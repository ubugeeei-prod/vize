"""Preserve exact instruction totals and direct call-edge counts by symbol."""

from collections import defaultdict
import json
from pathlib import Path
import re
import sys

lines = Path(sys.argv[1]).read_text().splitlines()
names = {}
for line in lines:
    match = re.match(r"(?:c?fn)=\((\d+)\) (.*)", line)
    if match:
        names[match[1]] = match[2]
exclusive = defaultdict(int)
calls = defaultdict(int)
current = None
callee = None
pending_call_cost = False
totals = None
for line in lines:
    if line.startswith("summary:"):
        totals = int(line.split()[1])
    if line.startswith("totals:") and totals == 0:
        totals = int(line.split()[1])
    match = re.match(r"fn=\((\d+)\)", line)
    if match:
        current = names.get(match[1], match[1])
        pending_call_cost = False
    match = re.match(r"cfn=\((\d+)\)", line)
    if match:
        callee = names.get(match[1], match[1])
    if line.startswith("calls="):
        calls[callee] += int(line.split()[0][6:])
        pending_call_cost = True
    elif re.match(r"[+*\-0-9]", line):
        if current and not pending_call_cost:
            exclusive[current] += int(line.split()[-1])
        pending_call_cost = False
assert totals is not None
assert sum(exclusive.values()) == totals, (sum(exclusive.values()), totals)
print(json.dumps({
    "instructions": totals,
    "exclusive_instructions_by_symbol": exclusive,
    "direct_calls_by_callee_symbol": calls,
}, indent=2))
