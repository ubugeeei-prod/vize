#!/usr/bin/env python3
"""Rank files by a composite readability score. Usage: worst.py <files.tsv> <generated.txt> <pinned-classified.tsv> [n]"""
import csv, sys
rows = list(csv.DictReader(open(sys.argv[1]), delimiter="\t"))
gen = set(open(sys.argv[2]).read().split())
pins = {r["path"]: r["class"] for r in csv.DictReader(open(sys.argv[3]), delimiter="\t")}
n = int(sys.argv[4]) if len(sys.argv) > 4 else 30
out = []
for r in rows:
    if r["lang"] == "md" or r["path"] in gen: continue
    v = {k: int(r[k] or 0) for k in r if k not in ("path", "area", "lang", "naming")}
    score = (max(0, v["lines"] - 500) / 100 + v["longest_fn"] / 80 + v["fns_over_80_no_blank"] * 0.5
             + v["redundant_lines"] / 8 + (v["hash_literals"] + v["repo_path_literals"]) / 25
             + v["magic_numbers"] / 60 + v["items_no_blank"] / 10 + len([x for x in r["naming"].split(",") if x and x != "codename-in-path"]))
    out.append((score, r["path"], v, r["naming"], pins.get(r["path"], "")))
out.sort(reverse=True)
print("| # | Score | Path | Lines | Longest fn | Fns>80 no-blank | Redundant | Magic | Path+hash lits | Naming | Pinned |")
print("|---|---|---|---|---|---|---|---|---|---|---|")
for i, (s, p, v, nm, pin) in enumerate(out[:n], 1):
    print(f"| {i} | {s:.1f} | `{p}` | {v['lines']} | {v['longest_fn']} | {v['fns_over_80_no_blank']} | {v['redundant_lines']} | {v['magic_numbers']} | {v['repo_path_literals']}+{v['hash_literals']} | {nm} | {pin} |")
