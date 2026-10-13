#!/usr/bin/env python3
"""Authenticate current compiled sources with exactly four historical overlays."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

if sys.flags.optimize:
    raise RuntimeError("optimized Python cannot authenticate source custody")

baseline, candidate, phase, destination = sys.argv[1:]
historical_before = "67e3d0e6775cc3a1f29bfb9309d991252f9e75ff"
integrated_after = "0f39ac834eebdb02821acd4bd81ce8d2a3c48bd1"
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
    "tests/tooling/custom-directive-source-custody.test.mjs",
    "docs/davinci/decisions/2026-10-12-integrated-custom-directive-custody.md",
}
neutral.update(
    "tests/_fixtures/differential/typechecker/dynamic-custom-directive-argument/" + name
    for name in ("App.vue.txt", "README.md", "cases.json", "original-custody.json", "tsconfig.json.txt")
)


def git(*args):
    return subprocess.check_output(["git", "--no-replace-objects", *args])


def tree(revision):
    result = {}
    for record in git("ls-tree", "-rz", "--full-tree", revision).split(b"\0"):
        if record:
            metadata, path = record.split(b"\t", 1)
            mode, kind, blob = metadata.decode().split()
            result[path.decode()] = (mode, kind, blob)
    return result


assert baseline == historical_before, ("unknown historical baseline", baseline)
assert git("rev-parse", "HEAD").decode().strip() == candidate, "candidate must equal HEAD"
assert phase in ("baseline", "candidate"), ("unknown counterfactual arm", phase)
git("merge-base", "--is-ancestor", integrated_after, candidate)
before, integrated, common = tree(historical_before), tree(integrated_after), tree(candidate)
for path in sorted(production):
    assert common.get(path) == integrated.get(path), ("integrated production body changed", path)
    assert before.get(path) != integrated.get(path), ("historical production delta missing", path)
expected = dict(common)
if phase == "baseline":
    for path in production:
        if path in before:
            expected[path] = before[path]
        else:
            expected.pop(path, None)


def compiled_path(path):
    return (path.startswith(("crates/", "davinci/", "vendor/", ".cargo/"))
            or path in ("Cargo.lock", "rust-toolchain.toml") or path.endswith("Cargo.toml"))


def member(path, entry):
    mode, kind, blob = entry
    assert kind == "blob", (path, kind)
    physical = Path(path)
    assert physical.is_symlink() == (mode == "120000"), ("source mode changed", path)
    if mode != "120000":
        assert physical.is_file(), ("source missing", path)
        assert bool(physical.stat().st_mode & 0o111) == (mode == "100755"), ("source mode changed", path)
    raw = os.readlink(path).encode() if mode == "120000" else physical.read_bytes()
    actual = hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()
    assert actual == blob, ("source body changed", path, actual, blob)
    return {"path": path, "mode": mode, "blob": blob, "bytes": len(raw),
            "sha256": hashlib.sha256(raw).hexdigest()}


compiled = []
for path, entry in sorted(expected.items()):
    if compiled_path(path):
        if entry[1] == "blob":
            compiled.append(member(path, entry))
        else:
            compiled.append({"path": path, "mode": entry[0], "kind": entry[1], "object": entry[2]})
for path in production - expected.keys():
    assert not os.path.lexists(path), ("historically absent production body exists", path)
untracked = git("ls-files", "--others", "--exclude-standard", "-z").split(b"\0")
assert not any(path and compiled_path(path.decode()) for path in untracked), "foreign compiled source"
driver = [member(path, common[path]) for path in sorted(neutral)]


def recorded_body(revision, path, entry):
    if entry is None:
        return {"path": path, "absent": True}
    mode, kind, blob = entry
    assert kind == "blob", (revision, path, kind)
    raw = git("cat-file", "blob", blob)
    return {"path": path, "mode": mode, "blob": blob, "bytes": len(raw),
            "sha256": hashlib.sha256(raw).hexdigest()}


packet = {
    "label": "current HEAD compiled tree + four historical production substitutions" if phase == "baseline"
             else "current HEAD compiled tree + integrated production bodies",
    "baseline": baseline, "candidate": candidate, "productionDelta": sorted(production),
    "counterfactualAuthority": {
        "phase": phase, "commonHead": candidate,
        "commonTree": git("rev-parse", f"{candidate}^{{tree}}").decode().strip(),
        "historicalBefore": historical_before, "integratedAfter": integrated_after,
        "historicalProduction": [recorded_body(historical_before, path, before.get(path))
                                 for path in sorted(production)],
        "integratedProduction": [recorded_body(integrated_after, path, integrated.get(path))
                                 for path in sorted(production)],
    },
    "compiledSourceManifest": compiled, "neutralDriverManifest": driver,
    "buildEnvironment": {name: os.environ.get(name) for name in (
        "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_TARGET_DIR", "CORSA_PATH", "VIZE_TEST_TSGO_PATH"
    )},
}
Path(destination).write_text(json.dumps(packet, indent=2) + "\n")
