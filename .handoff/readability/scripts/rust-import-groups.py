#!/usr/bin/env python3
"""Classify each Rust file's top-level `use` block: grouped std/external/local
with blank lines between groups, or mixed. Usage: rust-import-groups.py <root>..."""
import os, re, sys, collections
c = collections.Counter(); ex = collections.defaultdict(list)
for top in sys.argv[1:]:
    for dp, dn, fn in os.walk(top):
        if any(p in ('fixtures', 'snapshots', 'target') for p in dp.split('/')):
            dn[:] = []; continue
        for f in fn:
            if not f.endswith('.rs'): continue
            p = os.path.join(dp, f)
            L = open(p, encoding='utf-8', errors='replace').read().split('\n')
            mods = {m.group(1) for m in re.finditer(r'^(?:pub(?:\([\w:]+\))? )?mod (\w+)', '\n'.join(L), re.M)}
            groups, cur, seen = [], [], False
            for l in L:
                m = re.match(r'(?:pub(?:\([\w:]+\))? )?use (?:::)?(\w+)', l)
                if m:
                    seen = True; root = m.group(1)
                    cur.append('std' if root in ('std', 'core', 'alloc') else 'local' if root in ('crate', 'self', 'super') or root in mods else 'ext')
                    continue
                if seen and not l.strip():
                    if cur: groups.append(cur); cur = []
                    continue
                if seen and (l.startswith((' ', '}', '#['))): continue
                if seen: break
            if cur: groups.append(cur)
            if not groups: continue
            kinds = [set(g) for g in groups]
            if len(set().union(*kinds)) < 2: c['single-kind'] += 1; continue
            order_ok = all(['std','ext','local'].index(a) <= ['std','ext','local'].index(b) for g in groups for a, b in zip(g, g[1:]))
            key = ('mixed-in-one-group' if any(len(k) > 1 for k in kinds) else 'grouped-by-kind') + ('' if order_ok else '+unordered')
            c[key] += 1
            if len(ex[key]) < 3: ex[key].append(p)
print(dict(c)); print(dict(ex))
