#!/usr/bin/env python3
"""orphan-drivers.py stem... : driver files in .github/scripts/workflows/<stem>/ not reachable
from the generated .github/workflows/<stem>.yml (by file name, transitively). Run after regenerating."""
import os, sys, re
for stem in sys.argv[1:]:
    d=f'.github/scripts/workflows/{stem}'
    if not os.path.isdir(d): continue
    files=sorted(os.listdir(d))
    text=open(f'.github/workflows/{stem}.yml').read()
    # also reusable callers? only own yml
    reach=set(); frontier=[text]
    while frontier:
        t=frontier.pop()
        for f in files:
            if f not in reach and f in t:
                reach.add(f); frontier.append(open(os.path.join(d,f),errors='ignore').read())
    for f in files:
        if f not in reach: print(f'{d}/{f}')
