#!/usr/bin/env python3
"""Resolve conflict hunks in a list-of-workflows test file: keep the union of both sides'
entries, then drop entries (bracketed [..] items or single lines) naming a workflow file that
no longer exists in the working tree."""
import os, re, sys
p = sys.argv[1]
s = open(p).read()
def resolve(m):
    ours, theirs = m.group(1), m.group(2)
    lines = []
    for chunk in (ours, theirs):
        for line in chunk.splitlines(keepends=True):
            if line not in lines:
                lines.append(line)
    return "".join(lines)
s = re.sub(r"<<<<<<< [^\n]*\n(.*?)=======\n(.*?)>>>>>>> [^\n]*\n", resolve, s, flags=re.S)
# drop single-line entries and bracketed multi-line entries of missing workflows
def gone(text):
    return any(not os.path.exists(f) for f in re.findall(r"\.github/workflows/[a-z0-9-]+\.yml", text))
out = re.sub(r"\n( *)\[\n(?:\1  .*\n)*?\1\],(?=\n)", lambda m: "" if gone(m.group(0)) else m.group(0), s)
out = "\n".join(l for l in out.split("\n") if not (re.match(r"\s*\[.*\],\s*$", l) and gone(l)))
open(p, "w").write(out)
