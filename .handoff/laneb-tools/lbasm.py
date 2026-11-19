#!/usr/bin/env python3
"""List or diff disassembled functions from two disasm probe runs.

usage: lbasm.py <base-run> <head-run> <bench-file-stem> [function-substring]
Without a substring: list functions with instruction counts on both sides.
With one: print a unified diff of the first function whose name contains it.
"""
import difflib
import glob
import re
import sys

IC = '/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/laneb-ic'


def load(run, stem):
    path = glob.glob(f'{IC}/{run}/instruction-counts-*/disasm/{stem}.s')[0]
    funcs, name, fresh = {}, None, True
    for line in open(path, errors='replace').read().split('\n'):
        if not line.strip():
            fresh = True
            continue
        if fresh:
            name, fresh = line, False
            funcs.setdefault(name, [])
            continue
        funcs[name].append(re.sub(r'<[^>]*\+0x[0-9a-f]+>', '<+off>', line.strip()))
    return funcs


base_run, head_run, stem = sys.argv[1:4]
needle = sys.argv[4] if len(sys.argv) > 4 else None
b, h = load(base_run, stem), load(head_run, stem)
canon = lambda n: n.replace('vize_atelier_core::codegen::document::', 'vize_l4::write::document::')
hb = {canon(k): v for k, v in h.items()}
bb = {canon(k): v for k, v in b.items()}
if needle is None:
    for name in sorted(set(bb) | set(hb)):
        x, y = len(bb.get(name, [])), len(hb.get(name, []))
        mark = '' if x == y and bb.get(name) == hb.get(name) else '  *'
        print(f'{x:6d} {y:6d}{mark}  {name[:150]}')
else:
    names = [n for n in sorted(set(bb) | set(hb)) if needle in n]
    name = names[0]
    print('#', name, len(bb.get(name, [])), '->', len(hb.get(name, [])))
    for line in difflib.unified_diff(bb.get(name, []), hb.get(name, []), lineterm='', n=2):
        print(line)
