#!/usr/bin/env python3
"""rename-workflows.py <map.tsv> [old ...]

Rename workflows by stem (run from the repository root). map.tsv rows: old<TAB>new.
With extra arguments, only those old stems are renamed. Moves the Pkl source into the
group directory of the new prefix, the generated YAML (to keep its presentation), the
driver, Pkl-script and part directories and the byte-pin archive, then rewrites every
non-historical reference. Prints the files it rewrote.
"""
import os, re, subprocess, sys

GROUP_DIRS = {"01": "checks", "02": "release", "03": "performance", "04": "typecheck",
              "05": "editors", "06": "compiler", "07": "davinci", "08": "integrations",
              "09": "tooling", "10": "apps", "11": "maintenance"}
# Bare stems that are ordinary words; their quoted-stem uses are reviewed by hand.
GENERIC = {"check", "release", "fuzz", "miri", "e2e", "benchmark", "fresco", "zizmor"}
HISTORICAL = re.compile(r"^(docs/davinci/decisions/|docs/davinci/plan/phase-3-records/|docs/release/2026-|"
                        r"tests/_fixtures/|.*\.before\.yml\.txt$|.*-original-.*\.yml$|.*\.yml\.gz$|"
                        r"\.github/workflows/README\.md$)")


def git(*args):
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def mv(src, dst):
    if os.path.exists(src):
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        git("mv", src, dst)


rows = [line.rstrip("\n").split("\t") for line in open(sys.argv[1]) if line.strip()]
only = set(sys.argv[2:])
pairs = [(o, n) for o, n in rows if not only or o in only]
for old, new in pairs:
    sources = [p for p in git("ls-files", "tools/config/pkl/workflows").split() if p.endswith(f"/{old}.pkl")]
    assert len(sources) == 1, (old, sources)
    mv(sources[0], f"tools/config/pkl/workflows/{GROUP_DIRS[new[:2]]}/{new}.pkl")
    mv(f".github/workflows/{old}.yml", f".github/workflows/{new}.yml")
    mv(f".github/scripts/workflows/{old}", f".github/scripts/workflows/{new}")
    mv(f"tools/config/pkl/scripts/.github/scripts/workflows/{old}",
       f"tools/config/pkl/scripts/.github/scripts/workflows/{new}")
    mv(f"tools/config/pkl/parts/{old}", f"tools/config/pkl/parts/{new}")
    mv(f"tests/_fixtures/tooling/named-workflow-programs/{old}.yml.gz",
       f"tests/_fixtures/tooling/named-workflow-programs/{new}.yml.gz")

mapping = dict(pairs)
alternation = "|".join(re.escape(o) for o in sorted(mapping, key=len, reverse=True))
yml = re.compile(rf"(?<![\w.-])({alternation})\.yml\b")
directory = re.compile(rf"(scripts/workflows/|parts/|named-workflow-programs/)({alternation})(?=[/\"'`\s.)]|$)", re.M)
call = re.compile(rf"(stageWorkflowScripts|workflowProgram|readWorkflowShell|archivedWorkflow|workflowPredecessor)\((\s*)\"({alternation})\"")
escaped = re.compile(rf"(?<![\w.-])({alternation})\\\.yml\b")
history_key = re.compile(rf'^(  )"({alternation})": \{{', re.M)


def rewrite(text):
    text = yml.sub(lambda m: mapping[m.group(1)] + ".yml", text)
    text = directory.sub(lambda m: m.group(1) + mapping[m.group(2)], text)
    text = call.sub(lambda m: f'{m.group(1)}({m.group(2)}"{mapping[m.group(3)]}"', text)
    text = escaped.sub(lambda m: mapping[m.group(1)] + "\\.yml", text)
    return text


args = []
for old in mapping:
    args += ["-e", old]
candidates = subprocess.run(["git", "grep", "-lF", *args], capture_output=True, text=True).stdout.split("\n")
changed = []
for path in candidates:
    if not path or HISTORICAL.match(path) or not os.path.isfile(path):
        continue
    try:
        text = open(path, encoding="utf8").read()
    except (UnicodeDecodeError, IsADirectoryError):
        continue
    updated = rewrite(text)
    if path == "tools/support/ci/workflow-program-history.json":
        updated = history_key.sub(lambda m: f'{m.group(1)}"{mapping[m.group(2)]}": {{', updated)
    if updated != text:
        open(path, "w", encoding="utf8").write(updated)
        changed.append(path)
print("\n".join(changed))
