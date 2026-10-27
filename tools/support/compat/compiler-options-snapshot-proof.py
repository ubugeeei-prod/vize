"""Functional proof for borrowed compiler-option snapshots.

Runs the identical old-API lock observer against the immutable parent and the
candidate, then executes every selected existing chain test in both programs.
This is not a performance measurement or runtime Vue provider qualification.
"""

import atexit
import hashlib
import json
import os
import platform
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path.cwd()
OUT = Path(os.environ["COMPILER_OPTIONS_PROOF_OUT"])
BASE = "9dfc33388e704e321fca1cc2a4feffa818117bf8"
EXPECTED = os.environ["QUALIFICATION_HEAD_SHA"]
OBSERVER = Path("crates/vize_canon/tests/compiler_options_snapshot_io.rs")
CHAIN_FILTERS = ["tsconfig_extends", "compiler_snapshot", "compiler_inputs"]
OUT.mkdir(parents=True, exist_ok=True)
RESULT = {"scope": "source functional only; runtime Vue authority unproved",
          "baseline": BASE, "candidate": EXPECTED, "qualified": False,
          "processes": [], "testGroups": []}


def save_result():
    (OUT / "result.json").write_text(json.dumps(RESULT, indent=2) + "\n")


atexit.register(save_result)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def source_identity():
    commit = git("cat-file", "-p", "HEAD")
    return {"head": git("rev-parse", "HEAD"),
            "tree": git("rev-parse", "HEAD^{tree}"),
            "parents": re.findall(r"^parent (\w+)$", commit, re.MULTILINE),
            "status": git("status", "--porcelain"),
            "lockSha256": sha((ROOT / "Cargo.lock").read_bytes())}


def command(label, args):
    print(f"Running {label}", flush=True)
    started = time.monotonic_ns()
    timed_out = False
    try:
        process = subprocess.run(args, cwd=ROOT, capture_output=True, timeout=2100)
        status, stdout, stderr = process.returncode, process.stdout, process.stderr
    except subprocess.TimeoutExpired as error:
        timed_out = True
        status, stdout, stderr = None, error.stdout or b"", error.stderr or b""
    elapsed = time.monotonic_ns() - started
    (OUT / f"{label}.stdout").write_bytes(stdout)
    (OUT / f"{label}.stderr").write_bytes(stderr)
    executables = []
    output = (stdout + b"\n" + stderr).decode("utf-8", "replace")
    for name in re.findall(r"Running [^\n]* \(([^()\n]+)\)", output):
        path = ROOT / name
        if path.is_file():
            with path.open("rb") as image:
                digest = hashlib.file_digest(image, "sha256").hexdigest()
            executables.append({"path": str(path), "bytes": path.stat().st_size,
                                "sha256": digest})
    receipt = {"label": label, "command": args, "cwd": str(ROOT),
               "status": status, "timedOut": timed_out, "elapsedNs": elapsed,
               "stdoutBytes": len(stdout), "stderrBytes": len(stderr),
               "stdoutSha256": sha(stdout), "stderrSha256": sha(stderr),
               "executables": executables}
    (OUT / f"{label}.process.json").write_text(json.dumps(receipt, indent=2) + "\n")
    RESULT["processes"].append(receipt)
    save_result()
    assert not timed_out, f"{label} timed out; raw process bytes preserved"
    return status, output


def cargo(*args):
    return ["cargo", "test", "--locked", "--profile", "ci",
            "-p", "vize_canon", *args]


def summary(output):
    matches = re.findall(
        r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored;"
        r" (\d+) measured; (\d+) filtered out", output)
    assert len(matches) == 1, f"Expected one actually executed test packet: {matches}"
    state, passed, failed, ignored, measured, filtered = matches[0]
    return {"state": state, "passed": int(passed), "failed": int(failed),
            "ignored": int(ignored), "measured": int(measured),
            "filtered": int(filtered)}


def require_tests(label, args, expected_count=None):
    status, listing = command(label + "-list", args + ["--", "--list", "--format", "terse"])
    assert status == 0, f"{label} discovery failed"
    names = sorted(line[:-6] for line in listing.splitlines() if line.endswith(": test"))
    assert len(names) > 0 and len(names) == len(set(names)), f"{label} zero/duplicate tests"
    if expected_count is not None:
        assert len(names) == expected_count, (label, len(names), expected_count)
    status, output = command(
        label, args + ["--", "--nocapture", "--test-threads=1", "--color", "never"])
    packet = summary(output)
    executed = sorted(re.findall(r"^test (\S+) \.\.\. ok$", output, re.MULTILINE))
    assert status == 0 and packet["state"] == "ok", (label, status, packet)
    assert packet["passed"] == len(names) and packet["failed"] == packet["ignored"] == 0
    assert executed == names, (label, "actual test names differ", executed, names)
    RESULT["testGroups"].append({"label": label, "names": names, "summary": packet})
    return names


candidate_source = source_identity()
assert candidate_source["head"] == EXPECTED and candidate_source["status"] == ""
observer_bytes = (ROOT / OBSERVER).read_bytes()
(OUT / "compiler_options_snapshot_io.rs").write_bytes(observer_bytes)
(OUT / "proof-driver.py").write_bytes(Path(__file__).read_bytes())
RESULT["candidateSource"] = candidate_source
RESULT["observerSha256"] = sha(observer_bytes)
RESULT["host"] = {"platform": platform.platform(), "python": sys.version,
                  "functionalProfile": "ci", "cargoBuildJobs": os.environ["CARGO_BUILD_JOBS"],
                  "rustFlags": os.environ.get("RUSTFLAGS"),
                  "targetLinker": os.environ.get("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER")}
(OUT / "result.json").write_text(json.dumps(RESULT, indent=2) + "\n")
for label, args in [("rustc", ["rustc", "--version", "--verbose"]),
                    ("cargo", ["cargo", "--version"])]:
    status, _ = command(label, args)
    assert status == 0

before_names = {}
injected = False
try:
    git("checkout", "--detach", BASE)
    baseline_source = source_identity()
    assert baseline_source["head"] == BASE and baseline_source["status"] == ""
    assert "1f27dba30f1be1d2d9c445860797ce1523dc8f8b" in baseline_source["parents"]
    RESULT["baselineSource"] = baseline_source
    assert not (ROOT / OBSERVER).exists(), "Original source already owns observer path"
    (ROOT / OBSERVER).write_bytes(observer_bytes)
    injected = True
    assert git("diff", "--", "crates/vize_canon/src") == ""
    status, output = command(
        "before-cache-lock", cargo("--test", "compiler_options_snapshot_io",
                                  "--", "--nocapture", "--test-threads=1", "--color", "never"))
    packet = summary(output)
    assert status == 101 and packet == {
        "state": "FAILED", "passed": 0, "failed": 1, "ignored": 0,
        "measured": 0, "filtered": 0}, (status, packet)
    assert "reading compiler options created generated-project cache locks" in output
    RESULT["beforeLockFailure"] = packet
    for index, test_filter in enumerate(CHAIN_FILTERS):
        before_names[test_filter] = require_tests(
            f"before-chain-{index}", cargo("--lib", test_filter))
finally:
    if injected:
        assert (ROOT / OBSERVER).read_bytes() == observer_bytes
        (ROOT / OBSERVER).unlink()
    git("checkout", "--detach", EXPECTED)

assert source_identity() == candidate_source, "Candidate source did not restore exactly"
require_tests("candidate-observed",
              cargo("--lib", "compiler_options_snapshot::observed::tests"), 7)
require_tests("candidate-cache-lock",
              cargo("--test", "compiler_options_snapshot_io"), 1)
for index, test_filter in enumerate(CHAIN_FILTERS):
    names = require_tests(f"candidate-chain-{index}", cargo("--lib", test_filter))
    assert names == before_names[test_filter], "Original chain inventory changed"
assert source_identity() == candidate_source, "Candidate source changed during tests"
RESULT["qualified"] = True
RESULT["candidateSourceAfter"] = source_identity()
(OUT / "result.json").write_text(json.dumps(RESULT, indent=2) + "\n")
print(json.dumps({"qualified": True, "candidate": EXPECTED,
                  "testGroups": [{"label": group["label"], "count": len(group["names"])}
                                 for group in RESULT["testGroups"]]}, indent=2))
