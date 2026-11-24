#!/usr/bin/env python3
"""Find first-party files whose current bytes are pinned by hash somewhere in the tree.

Usage: python3 -I pins.py <checkout-root> <out-dir>

A file counts as pinned when its sha256 or its git blob id appears anywhere in the
tree outside upstream/forks. The script then classifies each pin:

  direct-in-test       the digest itself is spelled in a test/custody/transition source
  record-read-by-test  a record naming the file's path and digest is itself named by such a source
  record-only          only a data record (ledger, manifest, provenance JSON) carries the digest

Writes pinned.tsv (all), pinned-classified.tsv (first-party sources, fixtures excluded).
Needs ripgrep (`rg`) on PATH.
"""
import collections
import hashlib
import os
import re
import subprocess
import sys

root, out = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
os.makedirs(out, exist_ok=True)


def digests(width):
    result = subprocess.run(
        ["rg", "-o", "--no-filename", "-I", "-g", "!upstream/forks", rf"\b[0-9a-f]{{{width}}}\b", "."],
        cwd=root, capture_output=True, text=True,
    )
    return set(result.stdout.split())


h64, h40 = digests(64), digests(40)
exts = (".rs", ".ts", ".tsx", ".js", ".mjs", ".cjs", ".mts", ".vue", ".pkl", ".sh", ".md", ".toml",
        ".json", ".css", ".scss", ".lua", ".scm", ".yml", ".yaml", ".html", ".el", ".vim", ".nix")
fixture = re.compile(r"(^|/)(_fixtures|__fixtures__|fixtures|snapshots|__snapshots__|expected)/|^tests/(differential|performance)/")
pinned = []
for dp, dn, fn in os.walk(root):
    rel = os.path.relpath(dp, root)
    if rel.startswith(("upstream", ".git")) or "node_modules" in dp.split(os.sep):
        dn[:] = []
        continue
    for f in fn:
        if not f.endswith(exts):
            continue
        p = os.path.join(dp, f)
        data = open(p, "rb").read()
        sha = hashlib.sha256(data).hexdigest()
        blob = hashlib.sha1(b"blob %d\0" % len(data) + data).hexdigest()
        hits = [h for h in (sha,) if h in h64] + [h for h in (blob,) if h in h40]
        if hits:
            pinned.append((os.path.relpath(p, root), data.count(b"\n"), hits))

with open(os.path.join(out, "pinned.tsv"), "w") as o:
    for path, lines, hits in sorted(pinned):
        o.write(f"{path}\t{lines}\t{' '.join(hits)}\n")

sources = [r for r in pinned if not fixture.search(r[0])]
wanted = {h for _, _, hs in sources for h in hs}
pattern_file = os.path.join(out, "pin-digests.txt")
open(pattern_file, "w").write("\n".join(sorted(wanted)) + "\n")
refs = collections.defaultdict(set)
listing = subprocess.run(["rg", "-o", "-F", "-f", pattern_file, "-g", "!upstream/forks", "."],
                         cwd=root, capture_output=True, text=True).stdout
for line in listing.splitlines():
    file, digest = line.rsplit(":", 1)
    refs[digest].add(file.removeprefix("./"))

testish = re.compile(r"(\.test\.[cm]?[jt]s|custody[^/]*\.[cm]?[jt]s|transition[^/]*\.ts|/tests?/.*\.(rs|ts|mjs)|_tests?\.rs|tests\.rs)$")
blob_of_tests = []
for dp, dn, fn in os.walk(root):
    if "node_modules" in dp.split(os.sep) or os.path.relpath(dp, root).startswith(("upstream", ".git")):
        dn[:] = []
        continue
    for f in fn:
        rel = os.path.relpath(os.path.join(dp, f), root)
        if testish.search(rel):
            blob_of_tests.append(open(os.path.join(dp, f), encoding="utf-8", errors="replace").read())
tests_text = "\n".join(blob_of_tests)

with open(os.path.join(out, "pinned-classified.tsv"), "w") as o:
    o.write("path\tlines\tclass\treferenced_by\n")
    for path, lines, hits in sorted(sources):
        records = set().union(*(refs[h] for h in hits)) - {path}
        if any(h in tests_text for h in hits):
            kind = "direct-in-test"
        elif any(os.path.basename(r) in tests_text and path in open(os.path.join(root, r), encoding="utf-8", errors="replace").read() for r in records):
            kind = "record-read-by-test"
        else:
            kind = "record-only"
        o.write(f"{path}\t{lines}\t{kind}\t{' '.join(sorted(records))[:300]}\n")
