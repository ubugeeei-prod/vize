#!/usr/bin/env python3
"""Regroup the leading `use` block of Rust files into std / external / local
groups separated by one blank line, keeping each group's original order
(rustfmt sorts within groups afterwards). Files whose block holds comments are
skipped and listed. Usage: regroup_imports.py <file>..."""
import re
import sys

USE = re.compile(r"^(?:pub(?:\([\w:\s]+\))?\s+)?use\s+(?:::)?(\w+)")
MOD = re.compile(r"^(?:pub(?:\([\w:\s]+\))?\s+)?mod\s+(\w+)", re.M)


def kind(root, mods):
    if root in ("std", "core", "alloc"):
        return 0
    if root in ("crate", "self", "super") or root in mods:
        return 2
    return 1


def regroup(text):
    lines = text.split("\n")
    mods = set(MOD.findall(text))
    start = next((i for i, l in enumerate(lines) if USE.match(l)), None)
    if start is None:
        return None, "no use"
    # Attributes directly above the first use belong to it.
    while start > 0 and lines[start - 1].startswith("#["):
        start -= 1
    items, pending, pending_at, i = [], [], None, start
    while i < len(lines):
        l = lines[i]
        if not l.strip():
            i += 1
            continue
        if l.startswith("#["):
            if not pending:
                pending_at = i
            pending.append(l)
            i += 1
            continue
        m = USE.match(l)
        if not m:
            break
        stmt = [l]
        depth = l.count("{") - l.count("}")
        while not (stmt[-1].rstrip().endswith(";") and depth <= 0):
            i += 1
            if i >= len(lines):
                return None, "unterminated"
            stmt.append(lines[i])
            depth += lines[i].count("{") - lines[i].count("}")
        if any("//" in s or "/*" in s for s in stmt):
            return None, "comment in block"
        items.append((kind(m.group(1), mods), pending + stmt))
        pending = []
        i += 1
    # Attributes that belong to the next non-use item stay with it.
    end = pending_at if pending else i
    # Trim trailing blank lines from the block.
    while end > start and not lines[end - 1].strip():
        end -= 1
    block = lines[start:end]
    if any(l.strip().startswith(("//", "/*")) for l in block):
        return None, "comment in block"
    groups = []
    for k in (0, 1, 2):
        group = [line for kk, stmt in items if kk == k for line in stmt]
        if group:
            groups.append(group)
    new_block = []
    for g in groups:
        if new_block:
            new_block.append("")
        new_block.extend(g)
    if new_block == block:
        return None, "unchanged"
    return "\n".join(lines[:start] + new_block + lines[end:]), "regrouped"


for path in sys.argv[1:]:
    text = open(path, encoding="utf-8").read()
    out, why = regroup(text)
    if out is None:
        print(f"skip {why}: {path}")
        continue
    open(path, "w", encoding="utf-8").write(out)
    print(f"regrouped: {path}")
