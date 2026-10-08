"""One authorized finite experiment; never retries or selects successful counts."""

import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path
from archive import compare_units, feature_units, original_archive, select_tar
from envelope import compiler_envelope, elf_dependencies
from common import (ATTEMPTS, BRANCH, CASE, FEATURE, ORIGINAL_BLOBS, OVERLAY,
                    POSITIVE_CASE, SOURCES, command, git, require, sha256, write_json)
from verify import budget_trace, positive_trace, printed_rows


# The completed fdfe/9a diagnostic overlays this exact libtest source into the
# frozen b9/E1 manifests. A later standalone main is not a faithful overlay.
LIBTEST_BUDGET_OVERLAY_SHA256 = "c7dd7a9bbff3ddb3067ca13d6c5d4964103d49eb94798f18c069cded63e732dd"


def verify_budget_overlay(root):
    budget = root / "crates/vize_atelier_vapor/tests/davinci_vapor_native_budget.rs"
    require(sha256(budget) == LIBTEST_BUDGET_OVERLAY_SHA256,
            "historical libtest diagnostic budget overlay changed; reviewed recovery required")


def prepare_output(output, env):
    output.mkdir(parents=True, exist_ok=True)
    require({path.name for path in output.iterdir()} == {"preparation.json"},
            "only fresh preparation evidence admitted; no previous matrix reuse")
    value = json.loads((output / "preparation.json").read_text())
    expected = {"status": "preparation-only", "matrix_execution_credit": False,
                "matrix_attempts": 0, "head_sha": env["GITHUB_SHA"],
                "run_id": env["GITHUB_RUN_ID"], "run_attempt": env["GITHUB_RUN_ATTEMPT"]}
    require(type(value.get("matrix_attempts")) is int and value.get("matrix_execution_credit") is False,
            "literal integer zero and boolean false required")
    require(value == expected, "exact fresh preparation head/run/attempt and zero-credit identity")


def launch(binary, cwd, directory, threads, case, traced):
    directory.mkdir(parents=True)
    trace = directory / "trace.json"
    env = os.environ.copy()
    env.update(RUST_TEST_THREADS=str(threads), NEXTEST="1",
               NEXTEST_EXECUTION_MODE="process-per-test", RUST_BACKTRACE="1")
    env.pop("VIZE_ALLOCATION_TRACE_PATH", None)
    if traced:
        env["VIZE_ALLOCATION_TRACE_PATH"] = str(trace)
    env["LD_LIBRARY_PATH"] = str(binary.parent) + ":" + env.get("LD_LIBRARY_PATH", "")
    args = [str(binary), case, "--exact", "--nocapture"]
    started = time.time_ns()
    timed_out = False
    with (directory / "stdout.txt").open("wb") as stdout, (directory / "stderr.txt").open("wb") as stderr:
        try:
            result = subprocess.run(args, cwd=cwd, env=env, stdout=stdout, stderr=stderr, timeout=60)
            exit_code = result.returncode
        except subprocess.TimeoutExpired:
            exit_code, timed_out = None, True
    receipt = {"command": args, "cwd": str(cwd), "threads": threads,
               "started_ns": started, "ended_ns": time.time_ns(), "exit_code": exit_code,
               "timed_out": timed_out, "binary_sha256": sha256(binary),
               "stdout_sha256": sha256(directory / "stdout.txt"),
               "stderr_sha256": sha256(directory / "stderr.txt"), "trace_expected": traced,
               "trace_sha256": sha256(trace) if trace.exists() else None}
    # Store original status before any checker; failure evidence is never lost.
    write_json(directory / "execution.json", receipt)
    return receipt, (directory / "stdout.txt").read_text(), (directory / "stderr.txt").read_text()


def matrix_arm(binary, worktree, mode, source, output, attempts):
    roots = [parent for parent in binary.parents if (parent / "retained-members.json").exists()]
    require(len(roots) == 1, "one owning actual archive extraction manifest")
    selected = json.loads((roots[0] / "retained-members.json").read_text())
    authenticated = [row["sha256"] for row in selected if Path(row["member"]).name == binary.name]
    require(authenticated == [sha256(binary)], "selected ELF must retain its authenticated hash")
    for threads in (4, 1):
        for attempt in range(1, ATTEMPTS + 1):
            key = f"{source['label']}-{mode}-threads{threads}-attempt{attempt}"
            directory = output / "attempts" / key
            receipt, stdout, stderr = launch(binary, worktree / "crates/vize_atelier_vapor",
                                             directory, threads, CASE, mode == "instrumented")
            require(receipt["binary_sha256"] == authenticated[0], "actual executed authenticated ELF")
            record = {"key": key, "source_sha": source["sha"], "mode": mode,
                      "threads": threads, "attempt": attempt, **receipt}
            try:
                rows = printed_rows(stdout, stderr, receipt["exit_code"])
                joined = []
                if mode == "instrumented":
                    require((directory / "trace.json").exists(), "missing actual trace is fatal")
                    joined = budget_trace(json.loads((directory / "trace.json").read_text()), rows)
                record.update(complete=True, rows=rows, measured_windows=joined)
            except (RuntimeError, ValueError, KeyError, TypeError) as error:
                record.update(complete=False, diagnostic_error=str(error))
            write_json(directory / "observation.json", record)
            attempts.append(record)


def instrument(root, worktree, source, original_receipt, output):
    compiler_envelope(root, worktree, original_receipt, output)
    before = {}
    for path, blob in ORIGINAL_BLOBS.items():
        require(git(worktree, "rev-parse", f"HEAD:{path}") == blob, "immutable original source blob")
        before[path] = {"git_blob": blob, "sha256": sha256(worktree / path)}
    overlay = {}
    for path in OVERLAY:
        target = worktree / path
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(root / path, target)
        overlay[path] = sha256(target)
    require(git(worktree, "rev-parse", "HEAD") == source["sha"], "frozen build baseline")
    require(not (worktree / "target").exists(), "cold full workspace target tree required")
    for path in ("package.json", "pnpm-lock.yaml"):
        require(git(root, "rev-parse", f"HEAD:{path}") == git(worktree, "rev-parse", f"HEAD:{path}"),
                "shared locked JS build runtime must match frozen source")
    (worktree / "node_modules").symlink_to(root / "node_modules", target_is_directory=True)
    write_json(output / "overlay.json", {"source_sha": source["sha"], "before": before,
                                         "overlay_sha256": overlay})
    (output / "overlay.patch").write_text(git(worktree, "diff", "--", *ORIGINAL_BLOBS) + "\n")
    archive = worktree / "target/rust-test-archive/tests.tar.zst"
    archive.parent.mkdir(parents=True)
    args = ["cargo", "nextest", "archive", "--workspace", "--cargo-profile", "ci", "--timings",
            "--features", f"davinci_harness/{FEATURE}", "--archive-file", str(archive)]
    # Full workspace feature unification is preserved, never package-only.
    with (output / "build.stdout.txt").open("wb") as stdout, (output / "build.stderr.txt").open("wb") as stderr:
        result = subprocess.run(args, cwd=worktree, stdout=stdout, stderr=stderr)
    write_json(output / "build-execution.json", {"command": args, "exit_code": result.returncode,
        "source_sha": source["sha"], "compiler": command(["rustc", "-Vv"]),
        "nextest": command(["cargo", "nextest", "--version"]),
        "env": {key: os.environ.get(key) for key in
                ("RUSTFLAGS", "CARGO_BUILD_JOBS", "VIZE_TEST_REQUIRE_TSGO", "VIZE_NUXT_CONFIG_ITERATIONS")}})
    require(result.returncode == 0, "full workspace instrumented archive build failed")
    html = worktree / "target/cargo-timings/cargo-timing.html"
    shutil.copyfile(html, output / "cargo-timing.html")
    units = feature_units(html)
    write_json(output / "compiler-feature-inputs.json", units)
    write_json(output / "archive.json", {"source_sha": source["sha"],
        "archive_sha256": sha256(archive), "instrumented": True, "overlay": overlay})
    budget, harness = select_tar(archive, output / "selected")
    archive.unlink()
    return budget, harness, units


def experiment(root, output):
    require(sys.platform == "linux", "authorized diagnostic requires actual Linux TIDs")
    require(os.environ.get("GITHUB_REF") == f"refs/heads/{BRANCH}" and
            os.environ.get("GITHUB_EVENT_NAME") == "workflow_dispatch", "owned diagnostic dispatch only")
    # Reject an incompatible current source before fetching an old archive,
    # creating a worktree, querying tools or launching any matrix process.
    verify_budget_overlay(root)
    require(os.environ.get("VIZE_TEST_REQUIRE_TSGO") == "1" and
            os.environ.get("VIZE_NUXT_CONFIG_ITERATIONS") == "100" and
            "VIZE_TEST_DISABLE_TSGO" not in os.environ, "original full runtime environment")
    require(command(["rustc", "-V"]).startswith("rustc 1.98.0 ") and
            "0.9.146" in command(["cargo", "nextest", "--version"]), "pinned compiler/nextest")
    require(shutil.disk_usage(output).free >= 24 * 1024**3, "24 GiB free required for sequential builds")
    attempts, positives, identity = [], [], {"diagnostic_head": git(root, "rev-parse", "HEAD"),
        "sources": SOURCES, "attempts_per_arm": ATTEMPTS, "matrix_attempts": 64,
        "instrumentation_may_change_scheduling": True,
        "completeness_scope": "every exact measured global allocation ordinal",
        "acceptance_credit": False, "original_failure_preserved": True}
    write_json(output / "identity.json", identity)
    scratch = output.parent / "allocation-ownership-scratch"
    require(not scratch.exists(), "one finite matrix; never reuse a previous attempt directory")
    scratch.mkdir()
    for source in SOURCES:
        arm = output / source["label"]
        arm.mkdir()
        worktree = scratch / source["label"]
        subprocess.run(["git", "fetch", "--no-tags", "origin", source["sha"]], cwd=root, check=True)
        subprocess.run(["git", "worktree", "add", "--detach", str(worktree), source["sha"]], cwd=root, check=True)
        original, _, original_units = original_archive(source, arm / "original")
        original_receipt = json.loads((arm / "original/archive/receipt.json").read_text())
        require(git(worktree, "rev-parse", "HEAD^{tree}") ==
                original_receipt["tree"], "exact original tree")
        elf_dependencies(original, arm / "original/elf-dependencies.json")
        matrix_arm(original, worktree, "original", source, output, attempts)
        build = arm / "instrumented"
        build.mkdir()
        budget, harness, units = instrument(root, worktree, source, original_receipt, build)
        compare_units(original_units, units)
        elf_dependencies(budget, build / "budget-elf-dependencies.json")
        elf_dependencies(harness, build / "harness-elf-dependencies.json")
        receipt, stdout, _ = launch(harness, worktree / "tools/benchmarks/crates/davinci_harness",
                                    build / "positive-control", 4, POSITIVE_CASE, True)
        require((build / "positive-control/trace.json").exists(), "actual positive-control trace")
        positive = positive_trace(json.loads((build / "positive-control/trace.json").read_text()),
                                  stdout, receipt["exit_code"])
        positives.append({"source_sha": source["sha"], "windows": positive, **receipt})
        write_json(build / "positive-control/observation.json", positives[-1])
        matrix_arm(budget, worktree, "instrumented", source, output, attempts)
        # Remove only disposable owned runner worktrees/targets, not raw records.
        subprocess.run(["git", "worktree", "remove", "--force", str(worktree)], cwd=root, check=True)
    witnesses = [dict(attempt=attempt["key"], **window)
                 for attempt in attempts if attempt["complete"]
                 for window in attempt["measured_windows"] if window["lane"] == "native" and
                 window["global_calls"] > window["ceiling"] and
                 window["measuring_calls"] + window["other_worker_calls"] <= window["ceiling"] and
                 window["libtest_main_calls"] > 0]
    expected = {(source["sha"], mode, threads, attempt) for source in SOURCES
                for mode in ("original", "instrumented") for threads in (4, 1)
                for attempt in range(1, ATTEMPTS + 1)}
    observed = [(row["source_sha"], row["mode"], row["threads"], row["attempt"]) for row in attempts]
    require(len(observed) == len(set(observed)) and set(observed) == expected,
            "exact 64-arm cross-product; duplicates or substitutions are fatal")
    for source in SOURCES:
        for mode in ("original", "instrumented"):
            identities = {row["binary_sha256"] for row in attempts
                          if row["source_sha"] == source["sha"] and row["mode"] == mode}
            require(len(identities) == 1, "one immutable actual executable per source/mode")
    complete = all(attempt["complete"] for attempt in attempts) and len(positives) == 2
    summary = {**identity, "complete": complete, "attempts": attempts,
               "positive_controls": positives, "foreign_main_over_cap_witnesses": witnesses,
               "conclusion": "observed-libtest-main-over-cap" if complete and witnesses
                             else "cause-still-unknown", "cap_changes": False,
               "original_cap_failures": [attempt["key"] for attempt in attempts
                                         if attempt["exit_code"] == 101]}
    write_json(output / "summary.json", summary)
    require(complete, "all 64 attempts and both positive controls must be complete")
    return summary


if __name__ == "__main__":
    # Repository discovery must not start a Git process before overlay refusal.
    root = Path(__file__).resolve().parents[5]
    output = Path(sys.argv[1]).resolve()
    try:
        prepare_output(output, os.environ)
        result = experiment(root, output)
        print(json.dumps({key: result[key] for key in ("complete", "conclusion", "original_cap_failures")}))
    except Exception as error:
        fatal = {"complete": False, "cause": "still-unknown", "error": str(error),
                 "original_failure_preserved": True}
        # Existing evidence is never overwritten if this invocation is refused.
        with (output / f"fatal-{time.time_ns()}.json").open("x") as stream:
            stream.write(json.dumps(fatal, indent=2) + "\n")
        raise
