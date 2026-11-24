#!/usr/bin/env python3
"""move-rename-workflows.py <map.tsv> [old ...]   (run from the repository root, on top of J0)

One pass per workflow: move everything it owns into tools/ci/workflows/<new>/ and rewrite
every non-historical reference. map.tsv rows: old<TAB>new (new = NN-subject, or the
protected name itself, which keeps its output file name).

  tools/config/pkl/workflows/<area>/<old>.pkl                  -> tools/ci/workflows/<new>/workflow.pkl
  tools/config/pkl/parts/<old>/*.pkl                           -> tools/ci/workflows/<new>/jobs/*.pkl
  tools/config/pkl/scripts/.github/scripts/workflows/<old>/*   -> tools/ci/workflows/<new>/scripts/*  (Pkl recipes)
  .github/scripts/workflows/<old>/*                            -> tools/ci/workflows/<new>/scripts/*  (drivers + generated .sh)
  .github/workflows/<old>.yml                                  -> .github/workflows/<new>.yml (kept for presentation)
  tests/_fixtures/tooling/named-workflow-programs/<old>.yml.gz -> .../<new>.yml.gz

Relative Pkl imports are recomputed from the moved locations (pkl_move.py). Prints the
rewritten files.
"""
import os, re, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from pkl_move import move_files  # noqa: E402

HISTORICAL = re.compile(r"^(docs/davinci/decisions/|docs/release/2026-|tests/_fixtures/|tests/tooling/fixtures/|"
                        r".*\.before\.yml\.txt$|.*-original-.*$|.*\.yml\.gz$|\.github/workflows/README\.md$)")


def git(*args):
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def tracked(prefix):
    return [p for p in git("ls-files", "-z", "--", prefix).split("\0") if p]


rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1]) if l.strip()]
only = set(sys.argv[2:])
pairs = [(o, n) for o, n in rows if not only or o in only]
mapping = dict(pairs)

moves = {}
for old, new in pairs:
    home = f"tools/ci/workflows/{new}"
    sources = [p for p in tracked("tools/config/pkl/workflows") if p.endswith(f"/{old}.pkl")]
    assert len(sources) == 1, (old, sources)
    moves[sources[0]] = f"{home}/workflow.pkl"
    for f in tracked(f"tools/config/pkl/parts/{old}/"):
        moves[f] = f"{home}/jobs/" + f[len(f"tools/config/pkl/parts/{old}/"):]
    for root in (f"tools/config/pkl/scripts/.github/scripts/workflows/{old}/", f".github/scripts/workflows/{old}/"):
        for f in tracked(root):
            moves[f] = f"{home}/scripts/" + f[len(root):]
    moves[f".github/workflows/{old}.yml"] = f".github/workflows/{new}.yml"
    archive = f"tests/_fixtures/tooling/named-workflow-programs/{old}.yml.gz"
    if os.path.exists(archive):
        moves[archive] = f"tests/_fixtures/tooling/named-workflow-programs/{new}.yml.gz"
move_files(moves)
for top in ("tools/config/pkl/parts", "tools/config/pkl/scripts/.github/scripts/workflows",
            ".github/scripts/workflows", "tools/config/pkl/workflows"):
    for dirpath, _, _ in sorted(os.walk(top), reverse=True):
        if dirpath != top and not os.listdir(dirpath):
            os.rmdir(dirpath)

# Moved programs reach the shared ones from tools/ci/workflows/<name>/scripts/.
for old, new in pairs:
    scripts = f"tools/ci/workflows/{new}/scripts"
    for f in tracked(scripts):
        text = open(f).read()
        updated = text.replace("../../../../tools/ci/lib/scripts/", "../../../lib/scripts/")
        if updated != text:
            open(f, "w").write(updated)

alternation = "|".join(re.escape(o) for o in sorted(mapping, key=len, reverse=True))
rules = [
    (re.compile(rf"(?:tools/config/pkl/scripts/)?\.github/scripts/workflows/({alternation})(?=/|\"|'|`|\s|$)", re.M),
     lambda m: f"tools/ci/workflows/{mapping[m.group(1)]}/scripts"),
    (re.compile(rf"tools/config/pkl/workflows/[a-z]+/({alternation})\.pkl"),
     lambda m: f"tools/ci/workflows/{mapping[m.group(1)]}/workflow.pkl"),
    (re.compile(rf"tools/config/pkl/parts/({alternation})(?=/)"),
     lambda m: f"tools/ci/workflows/{mapping[m.group(1)]}/jobs"),
    (re.compile(rf"(named-workflow-programs/)({alternation})(?=\.yml\.gz)"),
     lambda m: m.group(1) + mapping[m.group(2)]),
    (re.compile(rf"(?<![\w.-])({alternation})\.yml\b"), lambda m: mapping[m.group(1)] + ".yml"),
    (re.compile(rf"(?<![\w.-])({alternation})\\\.yml\b"), lambda m: mapping[m.group(1)] + "\\.yml"),
    (re.compile(rf"stageWorkflowScripts\(\"({alternation})\"\)"),
     lambda m: f'stageWorkflow("{mapping[m.group(1)]}")'),
    (re.compile(rf"(workflowProgram|readWorkflowShell|archivedWorkflow|workflowPredecessor)\((\s*)\"({alternation})\""),
     lambda m: f'{m.group(1)}({m.group(2)}"{mapping[m.group(3)]}"'),
]
history_key = re.compile(rf'^(  )"({alternation})": \{{', re.M)

args = []
for old in mapping:
    args += ["-e", old]
changed = []
for path in subprocess.run(["git", "grep", "-lF", *args], capture_output=True, text=True).stdout.split("\n"):
    if not path or HISTORICAL.match(path) or not os.path.isfile(path):
        continue
    try:
        text = open(path, encoding="utf8").read()
    except UnicodeDecodeError:
        continue
    updated = text
    for pattern, replacement in rules:
        updated = pattern.sub(replacement, updated)
    if path == "tools/support/ci/workflow-program-history.json":
        updated = history_key.sub(lambda m: f'{m.group(1)}"{mapping[m.group(2)]}": {{', updated)
    if updated != text:
        open(path, "w", encoding="utf8").write(updated)
        changed.append(path)
print("\n".join(changed))
