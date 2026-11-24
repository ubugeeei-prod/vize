#!/usr/bin/env python3
"""Move Pkl modules (and any other files) with git mv, then rewrite every relative Pkl
import/amends/extends path in tools/ so each one still names the same target.

Usage as a module: move_files({old: new, ...}) -> list of rewritten files.
Run from the repository root.
"""
import os, re, subprocess

IMPORT = re.compile(r'((?:^|\s)(?:import\*?|amends|extends)\s+|import\*?\()"([^"]+)"', re.M)


def git(*args):
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def move_files(mapping):
    """mapping: repo-relative old path -> new path (files only)."""
    mapping = {os.path.normpath(o): os.path.normpath(n) for o, n in mapping.items()}
    before = [p for p in git("ls-files", "tools", "-z").split("\0") if p.endswith(".pkl")]
    for old, new in mapping.items():
        os.makedirs(os.path.dirname(new) or ".", exist_ok=True)
        git("mv", old, new)
    changed = []
    for old_path in before:
        new_path = mapping.get(os.path.normpath(old_path), old_path)
        if not os.path.exists(new_path):
            continue
        text = open(new_path).read()

        def fix(m):
            ref = m.group(2)
            if ref.startswith(("pkl:", "file:", "https:", "package:", "modulepath:")) or "*" in ref:
                return m.group(0)
            target = os.path.normpath(os.path.join(os.path.dirname(old_path), ref))
            target = mapping.get(target, target)
            rel = os.path.relpath(target, os.path.dirname(new_path))
            if not rel.startswith("."):
                rel = rel if "/" in rel or not ref.startswith("./") else "./" + rel
            return m.group(1) + '"' + rel + '"'

        updated = IMPORT.sub(fix, text)
        if updated != text:
            open(new_path, "w").write(updated)
            changed.append(new_path)
    return changed
