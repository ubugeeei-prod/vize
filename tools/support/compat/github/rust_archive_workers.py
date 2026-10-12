#!/usr/bin/env python3
"""Authenticate the frozen runtime and compare every original partition and result."""
import json
import os
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

from artifact_payload_files import digest_file, json_bytes, require
from rust_archive_transport import ARTIFACT, SOURCE, TREE, WORKSPACE, validate_receipt, verify

JUNIT = "source-results/junit.xml"


def command(*args):
    return subprocess.check_output(args, text=True).strip()


def pinned_nextest(banner):
    require(re.match(r"^cargo-nextest 0\.9\.146(?:\s|$)", banner), "runtime nextest release differs")


def frozen_test_environment(driver):
    source = command("git", "--no-replace-objects", "rev-parse", "HEAD")
    require(source == SOURCE and os.environ.get("GITHUB_SHA") == driver,
            "test source or orchestration identity differs")
    return {**os.environ, "GITHUB_SHA": source}


def context(custody):
    receipt = json.loads((Path(custody) / "original-receipt.json").read_bytes())
    validate_receipt(receipt)
    require(str(Path.cwd()) == WORKSPACE
            and command("git", "--no-replace-objects", "rev-parse", "HEAD") == SOURCE
            and command("git", "--no-replace-objects", "rev-parse", "HEAD^{tree}") == TREE,
            "frozen physical checkout identity differs")
    require(sys.platform == "linux" and os.uname().machine == "x86_64", "runtime platform differs")
    require(command("rustc", "-Vv") == receipt["rustcVersion"], "runtime compiler differs")
    pinned_nextest(command("cargo", "nextest", "--version"))
    expected = {"VIZE_TEST_DISABLE_TSGO": "1", "VIZE_TEST_REQUIRE_TSGO": None,
                "VIZE_NUXT_CONFIG_ITERATIONS": "100", "CARGO_BUILD_JOBS": "12",
                "RUST_TEST_THREADS": "4", "NEXTEST_PROFILE": "pr"}
    require(all(os.environ.get(key) == value for key, value in expected.items()),
            "original PR runtime envelope differs")
    require(not command("git", "--no-replace-objects", "status", "--porcelain", "--untracked-files=no"),
            "frozen tracked source is dirty")
    return {"sourceSha": SOURCE, "sourceTree": TREE, "environment": expected,
            "testGitHubSha": SOURCE,
            "rustcVersion": receipt["rustcVersion"], "nextestVersion": "0.9.146",
            "nextestConfigSha256": digest_file(".config/nextest.toml"),
            "cargoLockSha256": digest_file("Cargo.lock"), "packageLockSha256": digest_file("pnpm-lock.yaml"),
            "submodules": command("git", "--no-replace-objects", "submodule", "status")}


def record(root, arm, shard, driver, archive, custody, run_exit):
    root = Path(root)
    require(arm in ("baseline", "candidate") and shard in range(1, 5), "unknown worker vector")
    report = verify(custody, arm, archive, driver)
    runtime = json.loads((root / "runtime.json").read_bytes())
    require(runtime["sourceSha"] == SOURCE and runtime["sourceTree"] == TREE
            and runtime.get("testGitHubSha") == SOURCE, "worker source differs")
    result = {"schemaVersion": 1, "driverSha": driver, "artifactId": ARTIFACT,
              "sourceSha": SOURCE, "arm": arm, "shard": shard, "partition": f"hash:{shard}/4",
              "profile": "pr", "runExit": run_exit, "archiveSha256": report[arm]["archiveSha256"],
              "tarSha256": report[arm]["tarSha256"], "runtime": runtime,
              "driverFiles": report["driverFiles"],
              "testsSha256": digest_file(root / "tests.json"),
              "junitSha256": digest_file(root / JUNIT) if (root / JUNIT).exists() else None}
    (root / "worker.json").write_bytes(json_bytes(result))
    return result


def selected(listing):
    require(all(suite["binary-id"] == binary for binary, suite in listing["rust-suites"].items()),
            "original binary/suite identities differ")
    return {(binary, name) for binary, suite in listing["rust-suites"].items()
            for name, case in suite["testcases"].items()
            if case["filter-match"]["status"] == "matches"}


def outcomes(path, listing):
    root = ET.parse(path).getroot()
    result = []
    for suite in root.iter("testsuite"):
        for case in suite.findall("testcase"):
            result.append((suite.get("name"), case.get("classname"), case.get("name"),
                           tuple(child.tag for child in case if child.tag in ("failure", "error", "skipped"))))
    require(len(result) == len(set(item[:3] for item in result)), "actual JUnit case identity repeats")
    expected = {(binary, binary, name) for binary, name in selected(listing)}
    require(set(item[:3] for item in result) == expected,
            "actual JUnit case identities do not cover the complete selected original partition")
    require(all(not item[3] for item in result), "selected original cases did not actually pass")
    return sorted(result)


def aggregate(root, driver):
    workers = {}
    for path in Path(root).glob("*/worker.json"):
        receipt = json.loads(path.read_bytes())
        key = receipt["arm"], receipt["shard"]
        require(key not in workers, "worker repeats")
        require(receipt["schemaVersion"] == 1 and receipt["driverSha"] == driver
                and receipt["artifactId"] == ARTIFACT and receipt["sourceSha"] == SOURCE
                and receipt["runtime"].get("testGitHubSha") == SOURCE
                and receipt["profile"] == "pr" and receipt["partition"] == f"hash:{key[1]}/4"
                and receipt["runExit"] == 0, "worker provenance/result differs")
        folder = path.parent
        require(digest_file(folder / "tests.json") == receipt["testsSha256"]
                and digest_file(folder / JUNIT) == receipt["junitSha256"], "worker body differs")
        workers[key] = folder, receipt
    require(set(workers) == {(arm, shard) for arm in ("baseline", "candidate") for shard in range(1, 5)},
            "all four original partitions in both arms are required")
    reference = workers[("baseline", 1)][1]
    seen, comparisons = set(), []
    for shard in range(1, 5):
        baseline, before = workers[("baseline", shard)]
        candidate, after = workers[("candidate", shard)]
        require(before["runtime"] == after["runtime"] == reference["runtime"]
                and before["driverFiles"] == after["driverFiles"] == reference["driverFiles"]
                and before["tarSha256"] == after["tarSha256"] == reference["tarSha256"],
                "matched runtime/tar identity differs")
        left = json.loads((baseline / "tests.json").read_bytes())
        right = json.loads((candidate / "tests.json").read_bytes())
        require(left == right, "complete original partition listing differs")
        cases = selected(left)
        require(not seen.intersection(cases), "original hash partitions overlap")
        seen.update(cases)
        require(outcomes(baseline / JUNIT, left) == outcomes(candidate / JUNIT, right),
                "actual case/result vector differs")
        comparisons.append({"shard": shard, "selectedCases": len(cases),
                            "actualCases": len(outcomes(baseline / JUNIT, left))})
    full_before = json.loads((workers[("baseline", 1)][0] / "unpartitioned.json").read_bytes())
    full_after = json.loads((workers[("candidate", 1)][0] / "unpartitioned.json").read_bytes())
    require(full_before == full_after and seen == selected(full_before), "complete original PR vector is missing")
    result = {"schemaVersion": 1, "driverSha": driver, "sourceSha": SOURCE,
              "tarSha256": reference["tarSha256"], "selectedCases": len(seen), "partitions": comparisons,
              "qualification": "all four unchanged original PR-runtime partitions; no full-native claim"}
    (Path(root) / "aggregate.json").write_bytes(json_bytes(result))
    return result


if __name__ == "__main__":
    if sys.argv[1] == "execute":
        invocation = sys.argv[3:]
        require(invocation[:2] == ["cargo", "nextest"]
                and len(invocation) > 2 and invocation[2] in ("list", "run"),
                "only the original nextest list and run commands are permitted")
        os.execvpe(invocation[0], invocation, frozen_test_environment(sys.argv[2]))
    elif sys.argv[1] == "context":
        result = context(sys.argv[2])
        Path(sys.argv[3]).write_bytes(json_bytes(result))
    elif sys.argv[1] == "record":
        result = record(sys.argv[2], sys.argv[3], int(sys.argv[4]), *sys.argv[5:8], int(sys.argv[8]))
    elif sys.argv[1] == "aggregate":
        result = aggregate(*sys.argv[2:])
    else:
        raise ValueError("unknown worker command")
    print(json.dumps(result, sort_keys=True))
