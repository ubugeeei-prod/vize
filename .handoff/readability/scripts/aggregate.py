#!/usr/bin/env python3
"""Roll files.tsv (from inventory.py) up into the inventory.md tables.

Usage: python3 -I aggregate.py <files.tsv> [generated-files.txt] > tables.md

Files listed in generated-files.txt (header says "generated"/"do not edit") are dropped.
"""
import csv
import sys
from collections import defaultdict

rows = list(csv.DictReader(open(sys.argv[1]), delimiter="\t"))
if len(sys.argv) > 2:
    generated = set(open(sys.argv[2]).read().split())
    rows = [r for r in rows if r["path"] not in generated]
NUM = [k for k in rows[0] if k not in ("path", "area", "lang", "naming")]
for r in rows:
    for k in NUM:
        r[k] = int(r[k] or 0)
    r["naming"] = [n for n in r["naming"].split(",") if n]


def group(r):
    top = r["path"].split("/")[0]
    if top in ("playground", "docs", "examples"):
        return "apps"
    return top


def table(title, key_fn, rows, order=None, min_files=1):
    agg = defaultdict(lambda: defaultdict(int))
    for r in rows:
        a = agg[key_fn(r)]
        a["files"] += 1
        a["lines"] += r["lines"]
        a["nonblank"] += r["lines"] - r["blank"]
        a["over500"] += r["lines"] > 500
        a["over1000"] += r["lines"] > 1000
        for k in ("comment_lines_plain", "comment_lines_doc", "redundant_lines", "r_issue", "r_history",
                  "r_restates", "r_banner", "r_commented_code", "magic_numbers", "repo_path_literals",
                  "hash_literals", "issue_idents", "fns_over_80", "fns_over_80_no_blank", "long_lines_100",
                  "items_no_blank"):
            a[k] += r[k]
        a["longest_fn"] = max(a["longest_fn"], r["longest_fn"])
        a["naming"] += len([n for n in r["naming"] if n != "codename-in-path"])
        a["codename"] += "codename-in-path" in r["naming"]
    print(f"\n### {title}\n")
    print("| Area | Files | Lines | >500 | >1000 | Comment % (plain/doc) | Redundant est. (lines, % of comments) | Issue refs | History | Restates | Banners | Commented code | Magic nums | Repo-path lits | Hash lits | Fns >80 (no blank) | Longest fn | Naming flags | Codename paths |")
    print("|" + "---|" * 20)
    keys = order or sorted(agg, key=lambda k: -agg[k]["lines"])
    for k in keys:
        a = agg[k]
        if a["files"] < min_files:
            continue
        com = a["comment_lines_plain"] + a["comment_lines_doc"]
        nb = max(1, a["nonblank"])
        red_pct = 100 * a["redundant_lines"] / com if com else 0
        print(f"| `{k}` | {a['files']} | {a['lines']:,} | {a['over500']} | {a['over1000']} | "
              f"{100*a['comment_lines_plain']/nb:.1f} / {100*a['comment_lines_doc']/nb:.1f} | "
              f"{a['redundant_lines']:,} ({red_pct:.0f}%) | {a['r_issue']:,} | {a['r_history']:,} | {a['r_restates']:,} | "
              f"{a['r_banner']:,} | {a['r_commented_code']:,} | {a['magic_numbers']:,} | {a['repo_path_literals']:,} | "
              f"{a['hash_literals']:,} | {a['fns_over_80']} ({a['fns_over_80_no_blank']}) | {a['longest_fn']} | {a['naming']} | {a['codename']} |")


def lang_table(rows):
    agg = defaultdict(lambda: defaultdict(int))
    for r in rows:
        a = agg[r["lang"]]
        a["files"] += 1
        a["lines"] += r["lines"]
        a["over500"] += r["lines"] > 500
        a["over1000"] += r["lines"] > 1000
    print("\n### By language\n\n| Language | Files | Lines | >500 | >1000 |\n|---|---|---|---|---|")
    for k in sorted(agg, key=lambda k: -agg[k]["lines"]):
        a = agg[k]
        print(f"| {k} | {a['files']:,} | {a['lines']:,} | {a['over500']} | {a['over1000']} |")


code = [r for r in rows if r["lang"] != "md"]
lang_table(rows)
table("By top-level group (code files, Markdown excluded)", group, code)
for top in ("crates", "davinci", "npm", "tools", "tests", "editors"):
    table(f"{top}/*", lambda r: r["area"], [r for r in code if r["path"].startswith(top + "/")])
table("apps (playground, docs code, examples)", lambda r: "/".join(r["path"].split("/")[:2]),
      [r for r in code if group(r) == "apps"])
table("root files", lambda r: r["path"], [r for r in code if "/" not in r["path"]])
