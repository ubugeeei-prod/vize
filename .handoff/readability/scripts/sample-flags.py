#!/usr/bin/env python3
"""Print a random sample of comment blocks the inventory heuristics flag, for precision checks.
Usage: sample-flags.py <checkout-root> <flag: restates|history|issue|banner|commented_code> <path-prefix> [n] [seed]"""
import os, random, re, sys
sys.argv, args = sys.argv[:3], sys.argv[1:]
sys.path.insert(0, os.path.dirname(__file__))
import inventory as inv
root, flag, prefix = args[0], args[1], args[2]
n = int(args[3]) if len(args) > 3 else 15
random.seed(int(args[4]) if len(args) > 4 else 1)
hits = []
for dp, dn, fn in os.walk(os.path.join(root, prefix)):
    for f in fn:
        path = os.path.relpath(os.path.join(dp, f), root)
        if not inv.in_scope(path) or os.path.splitext(f)[1] not in ('.rs', '.ts', '.js', '.mjs'):
            continue
        lang = 'rust' if f.endswith('.rs') else 'ts'
        text = open(os.path.join(dp, f), encoding='utf-8', errors='replace').read()
        comments, code = inv.split_c_like(text, lang)
        blocks, cur = [], []
        for c in comments:
            if cur and (c[0] != cur[-1][0] + 1 or c[1] != cur[-1][1]): blocks.append(cur); cur = []
            cur.append(c)
        if cur: blocks.append(cur)
        nb = {k: v for k, v in code.items() if v.strip()}
        for b in blocks:
            t = ' '.join(x[2] for x in b)
            nxt = nb.get(b[-1][0] + 1)
            ok = {'restates': len(b) <= 2 and inv.restates(t, nxt), 'history': any(inv.HISTORY_RE.search(x[2]) for x in b),
                  'issue': any(inv.ISSUE_RE.search(x[2]) for x in b), 'banner': all(inv.BANNER_RE.match(x[2]) or not x[2] for x in b),
                  'commented_code': b[0][1] != 'doc' and any(inv.CODEISH_RE.match(x[2]) for x in b if x[2])}[flag]
            if ok and not inv.KEEP_RE.match(t): hits.append((path, b[0][0], t[:140], (nxt or '').strip()[:80]))
print(len(hits), 'hits')
for h in random.sample(hits, min(n, len(hits))): print(f'{h[0]}:{h[1]}  // {h[2]}\n      -> {h[3]}')
