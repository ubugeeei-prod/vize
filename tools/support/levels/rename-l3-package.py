#!/usr/bin/env python3
"""Rewrite the L3 package identity after its directory's move-only commit.

Run with --repo pointing to an isolated checkout, then run cargo fmt --all.
The immutable semver baseline is deliberately a separate reviewed change.
"""

import argparse
import json
from pathlib import Path
import re
import subprocess

CURRENT_DOCS = {
    "docs/davinci/architecture.md",
    "docs/davinci/layer-names.md",
    "docs/davinci/open-questions.md",
    "docs/davinci/plan/impeto-ops.md",
    "docs/davinci/plan/test-suites.md",
    "docs/davinci/no-std-boundary.md",
    "docs/davinci/plan/storage-inventory.tsv",
}


def reorder_l3_lockfile(source):
    chunks = source.split("\n[[package]]\n")
    prefix = chunks.pop(0)
    for index, chunk in enumerate(chunks):
        match = re.search(
            r"^dependencies = \[\n(?P<body>.*?)^\]", chunk, re.M | re.S
        )
        if match and ' "vize_l3",' in match.group("body"):
            body = "\n".join(sorted(match.group("body").splitlines())) + "\n"
            chunks[index] = chunk[:match.start("body")] + body + chunk[match.end("body"):]
    renamed = [chunk for chunk in chunks if re.search(r'^name = "vize_l3"$', chunk, re.M)]
    assert len(renamed) == 1, "expected exactly one L3 package"
    chunks = [chunk for chunk in chunks if chunk not in renamed]
    position = next(
        index for index, chunk in enumerate(chunks)
        if re.search(r'^name = "([^"]+)"$', chunk, re.M)[1] > "vize_l3"
    )
    chunks[position:position] = renamed
    return prefix + "\n[[package]]\n" + "\n[[package]]\n".join(chunks)


def update_metadata_assertions(name, source):
    if name in {
        "tests/tooling/davinci-stage-dependencies.test.ts",
        "tests/tooling/davinci-ssr-l4-bridge.test.ts",
    }:
        source = source.replace('["vize_l3", "vize_l3"]', '["vize_l3", null]')
    if name == "tests/tooling/davinci-stage-dependencies.test.ts":
        source = source.replace(
            '["vize_carton", "vize_davinci", "vize_l3", "vize_l1", "vize_l2"]',
            '["vize_carton", "vize_davinci", "vize_l1", "vize_l2", "vize_l3"]',
        ).replace(
            '["vize_carton", "vize_davinci", "vize_l3", "vize_l2"]',
            '["vize_carton", "vize_davinci", "vize_l2", "vize_l3"]',
        )
        source = source.replace('package = "vize_l3", ', "")
        source = source.replace(
            "Davinci L3 uses the Impeto package through the stage alias",
            "Davinci L3 uses the physical crate package and directory",
        ).replace(
            'assert.doesNotMatch(workspaceManifest, /^vize_l3 = \\{ path = "crates\\/vize_l3"/m);',
            'assert.doesNotMatch(workspaceManifest, /\\bvize_impeto\\b/u);',
        )
        source = source.replace("const impetoManifest =", "const l3Manifest =")
        source = source.replace("assert.match(impetoManifest,", "assert.match(l3Manifest,")
    if name == "tests/tooling/davinci-stage-release-firewall.test.ts":
        source = source.replace('rename: "vize_l3"', "rename: null")
        block = (
            '    {\n      name: "vize_l3",\n'
            '      req: versionRequirement("vize_l3"),\n'
            '      rename: null,\n      optional: false,\n      features: [],\n    },\n'
        )
        last = block.replace('"vize_l3"', '"vize_l2_to_l3"')
        assert source.count(block) == 2 and source.count(last) == 2
        source = source.replace(block, "").replace(last, last + block)
    return source


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, type=Path)
    repo = parser.parse_args().repo.resolve()
    if not (repo / "crates/vize_l3/Cargo.toml").is_file():
        raise SystemExit("move-only commit must exist first")
    if "vize_impeto" not in (repo / "Cargo.toml").read_text():
        print(json.dumps({"filesChanged": 0, "status": "physical identity already renamed"}))
        return
    names = subprocess.check_output(
        ["git", "-C", str(repo), "ls-files"], text=True
    ).splitlines()
    changes = []
    for name in names:
        if name.startswith("docs/") and name not in CURRENT_DOCS:
            continue
        if name == "tools/commands/ci/github/semver-baseline.rs":
            continue
        path = repo / name
        if not path.is_file() or path.resolve() == Path(__file__).resolve():
            continue
        try:
            before = path.read_text()
        except UnicodeDecodeError:
            continue
        if "vize_impeto" not in before:
            continue
        after = before
        if name == "Cargo.toml":
            after = re.sub(r'^vize_impeto = \{[^\n]+\}\n', "", after, flags=re.M)
        after = after.replace("vize_impeto", "vize_l3")
        if name.endswith("Cargo.toml"):
            after = after.replace('package = "vize_l3", ', "")
        if name == "Cargo.lock":
            after = reorder_l3_lockfile(after)
        after = update_metadata_assertions(name, after)
        if before != after:
            path.write_text(after)
            changes.append(name)
    print(json.dumps({"filesChanged": len(changes), "paths": changes}, indent=2))


if __name__ == "__main__":
    main()
