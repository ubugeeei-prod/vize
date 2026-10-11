#!/usr/bin/env python3
"""Authenticate the bounded production replay and its unchanged neutral driver."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

baseline, candidate, phase, destination = sys.argv[1:]
production = {
    "crates/vize_canon/src/virtual_ts/expressions/directive_values.rs",
    "crates/vize_canon/src/virtual_ts/expressions/directive_values/arguments.rs",
    "crates/vize_canon/src/virtual_ts/expressions/directive_values/presence.rs",
    "crates/vize_canon/src/virtual_ts/expressions/statements.rs",
}
neutral = {
    ".github/workflows/typechecker-native-phases.yml",
    "crates/vize_canon/tests/native_custom_directive_arguments.rs",
    "docs/davinci/decisions/2026-09-27-level-restructure.md",
    "docs/davinci/decisions/2026-10-11-custom-directive-argument-values.md",
    "docs/davinci/plan/consumer-migration-surfaces/typechecker/vize_canon.tsv",
    "tools/support/compat/github/native-custom-directive-arguments.sh",
    "tools/support/compat/github/custom-directive-source-custody.py",
}
neutral.update(
    "tests/_fixtures/differential/typechecker/dynamic-custom-directive-argument/" + name
    for name in ("App.vue.txt", "README.md", "cases.json", "original-custody.json", "tsconfig.json.txt")
)


def git(*args):
    return subprocess.check_output(["git", *args])


def tree(revision):
    result = {}
    for record in git("ls-tree", "-rz", "--full-tree", revision).split(b"\0"):
        if record:
            metadata, path = record.split(b"\t", 1)
            mode, kind, blob = metadata.decode().split()
            result[path.decode()] = (mode, kind, blob)
    return result


assert git("rev-parse", "HEAD").decode().strip() == candidate
before, after = tree(baseline), tree(candidate)
changed = {path for path in before.keys() | after.keys() if before.get(path) != after.get(path)}
assert changed - neutral == production, sorted(changed - neutral)
assert phase in ("baseline", "candidate")
expected = before if phase == "baseline" else after


def member(path, entry):
    mode, kind, blob = entry
    assert kind == "blob", (path, kind)
    raw = os.readlink(path).encode() if mode == "120000" else Path(path).read_bytes()
    actual = hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()
    assert actual == blob, (path, actual, blob)
    return {"path": path, "mode": mode, "blob": blob, "bytes": len(raw),
            "sha256": hashlib.sha256(raw).hexdigest()}


compiled = []
for path, entry in sorted(expected.items()):
    if path not in neutral and (
        path.startswith(("crates/", "davinci/", "vendor/", ".cargo/"))
        or path in ("Cargo.lock", "rust-toolchain.toml")
        or path.endswith("Cargo.toml")
    ):
        if entry[1] == "blob":
            compiled.append(member(path, entry))
        else:
            compiled.append({"path": path, "mode": entry[0], "kind": entry[1], "object": entry[2]})
for path in production - expected.keys():
    assert not Path(path).exists(), path
driver = [member(path, after[path]) for path in sorted(changed & neutral)]
packet = {
    "label": f"{baseline[:7]} production + candidate neutral test driver" if phase == "baseline"
             else "candidate production + candidate neutral test driver",
    "baseline": baseline, "candidate": candidate, "productionDelta": sorted(production),
    "compiledSourceManifest": compiled, "neutralDriverManifest": driver,
    "buildEnvironment": {name: os.environ.get(name) for name in (
        "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_TARGET_DIR", "CORSA_PATH", "VIZE_TEST_TSGO_PATH"
    )},
}
Path(destination).write_text(json.dumps(packet, indent=2) + "\n")
