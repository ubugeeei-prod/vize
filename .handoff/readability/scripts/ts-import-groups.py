#!/usr/bin/env python3
"""Classify each TS/JS file's leading import block: builtin (node:), package,
relative. Reports grouped-with-blank-lines vs mixed, and `import type` style.
Usage: ts-import-groups.py <root>..."""
import os, re, sys, collections
c = collections.Counter(); ex = collections.defaultdict(list)
skip = {'node_modules', 'fixtures', '_fixtures', '__fixtures__', 'snapshots', '__snapshots__', 'dist', 'target', 'test-fixtures'}
for top in sys.argv[1:]:
    for dp, dn, fn in os.walk(top):
        dn[:] = [d for d in dn if d not in skip]
        for f in fn:
            if not re.search(r'\.(?:[cm]?[jt]sx?)$', f) or f.endswith('.d.ts'): continue
            p = os.path.join(dp, f)
            text = open(p, encoding='utf-8', errors='replace').read()
            groups, cur = [], []
            for stmt in re.finditer(r'^(import\b[^;]*?from\s+["\']([^"\']+)["\'];?|import\s+["\']([^"\']+)["\'];?|\s*$|//.*$)', text, re.M):
                pass
            lines = text.split('\n'); i = 0; started = False
            while i < len(lines):
                l = lines[i]
                if l.startswith('import '):
                    started = True
                    stmt = l
                    while not re.search(r'["\'][^"\']+["\'];?\s*$', stmt) and i + 1 < len(lines):
                        i += 1; stmt += lines[i]
                    m = re.search(r'["\']([^"\']+)["\'];?\s*$', stmt)
                    spec = m.group(1) if m else ''
                    kind = 'builtin' if spec.startswith('node:') else 'relative' if spec.startswith('.') else 'package'
                    cur.append(kind)
                    if stmt.startswith('import type'): c['import-type-statements'] += 1
                    if re.search(r'\{[^}]*\btype\s+\w', stmt): c['inline-type-specifiers'] += 1
                elif started and not l.strip():
                    if cur: groups.append(cur); cur = []
                elif started and l.startswith(('//', '/*', ' *')):
                    pass
                elif started:
                    break
                i += 1
            if cur: groups.append(cur)
            if not groups: continue
            allk = set(k for g in groups for k in g)
            if len(allk) < 2: c['single-kind'] += 1; continue
            key = 'mixed-in-one-group' if any(len(set(g)) > 1 for g in groups) else 'grouped-by-kind'
            c[key] += 1
            if len(ex[key]) < 3: ex[key].append(p)
print(dict(c)); print(dict(ex))
