"""Compare source-copy admission receipts without treating profiler timings as normal timings."""

import copy
import hashlib
import json
from pathlib import Path
import statistics
import sys


def sample(path):
    row = json.loads(path.read_text())
    assert type(row["iterations"]) is int and row["iterations"] > 0
    assert type(row["sourceBytes"]) is int and row["sourceBytes"] > 0
    assert row["sourceBytes"] >= row["minimumSourceBytes"]
    return row


def signature(row):
    return tuple(row[key] for key in (
        "iterations", "sourceBytes", "sourceHash", "rows", "scenario", "checksum",
    ))


def count(row, name):
    entries = [entry for entry in row["micro"] if entry["name"] == name]
    assert len(entries) <= 1, (name, entries)
    for entry in entries:
        assert all(type(entry[key]) is int and entry[key] >= 0 for key in ("calls", "units"))
    return (entries[0]["calls"], entries[0]["units"]) if entries else (0, 0)


def profile_pair(left, right):
    assert signature(left) == signature(right)
    assert left["instrumented"] and right["instrumented"]
    iterations = left["iterations"]
    assert type(iterations) is int and iterations > 0
    assert left["scenario"] in ("irreparable", "valid", "repairable")
    for row in (left, right):
        assert count(row, "croquis.oxc.original.bytes") == (
            iterations, iterations * row["sourceBytes"],
        )
        expected_recoveries = iterations if row["scenario"] == "repairable" else 0
        assert count(row, "croquis.oxc.recovery.bytes") == (
            expected_recoveries, expected_recoveries * row["sourceBytes"],
        )
    # Compare semantic source counters, independent of changed source line numbers.
    semantic_counts = lambda row: sorted(
        (entry["name"], entry["calls"], entry["units"])
        for entry in row["micro"] if entry["name"].startswith("croquis.")
    )
    assert semantic_counts(left) == semantic_counts(right)
    allocation = {role: row["allocation"] for role, row in (("base", left), ("head", right))}
    calls_saved = allocation["base"]["calls"] - allocation["head"]["calls"]
    bytes_saved = allocation["base"]["requestedBytes"] - allocation["head"]["requestedBytes"]
    if left["scenario"] == "irreparable":
        assert calls_saved == iterations, allocation
        assert bytes_saved >= iterations * left["sourceBytes"], allocation
        assert bytes_saved % iterations == 0, allocation
    else:
        assert calls_saved == 0 and bytes_saved == 0, allocation
    return {
        "scope": "instrumented successful Rust allocation requests; cumulative layout traffic, not copied/live/RSS bytes",
        "base": allocation["base"], "head": allocation["head"],
        "callsSaved": calls_saved, "requestedBytesSaved": bytes_saved,
        "savedLayoutBytesPerParse": bytes_saved // iterations,
        "originalParsesPerRole": iterations,
        "recoveryParsesPerRole": iterations if left["scenario"] == "repairable" else 0,
        "croquisSourceCountsEqual": True,
    }


def profile_identity(root):
    receipt = json.loads((root / "identity.json").read_text())
    assert receipt["base"] == receipt["firstParent"]
    records = {}
    for role in ("base", "head"):
        normal = receipt["roles"][role]
        source = Path(normal["sourceRoot"])
        assert all(not artifact["fresh"] for artifact in normal["artifacts"])
        assert "micro-profiling" not in normal["artifacts"][1]["features"]
        messages = [
            json.loads(line) for line in (root / f"profile-build-{role}.jsonl").read_text().splitlines()
        ]
        artifacts = [row for row in messages if row.get("reason") == "compiler-artifact"]
        croquis = [
            row for row in artifacts
            if row["target"]["name"] == "vize_croquis" and row["target"]["kind"] == ["lib"]
        ]
        carriers = [
            row for row in artifacts
            if row["target"]["name"] == "source_recovery_clone_benchmark"
            and row["target"]["kind"] == ["example"]
        ]
        assert len(croquis) == len(carriers) == 1
        assert Path(croquis[0]["manifest_path"]) == source / "crates/vize_croquis/Cargo.toml"
        assert Path(carriers[0]["target"]["src_path"]) == source / "davinci/vize_l0/examples/source_recovery_clone_benchmark.rs"
        assert "micro-profiling" in carriers[0]["features"]
        assert not croquis[0]["fresh"] and not carriers[0]["fresh"]
        executable = Path(carriers[0]["executable"])
        sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
        assert sha(executable) == sha(root / f"instrumented-{role}")
        records[role] = {
            "sourceRoot": str(source),
            "binarySha256": sha(executable),
            "artifacts": [
                {key: row[key] for key in ("package_id", "manifest_path", "target", "profile", "features", "fresh", "executable")}
                for row in (croquis[0], carriers[0])
            ],
        }
    assert records["base"]["binarySha256"] != records["head"]["binarySha256"]
    for index in (0, 1):
        for key in ("features", "profile"):
            assert records["base"]["artifacts"][index][key] == records["head"]["artifacts"][index][key]
            assert receipt["roles"]["base"]["artifacts"][index][key] == receipt["roles"]["head"]["artifacts"][index][key]
    receipt["instrumentedRoles"] = records
    (root / "identity.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt


def self_test():
    def fixture(scenario, calls, allocated):
        recovery = 4 if scenario == "repairable" else 0
        return {
            "iterations": 4, "sourceBytes": 100, "sourceHash": 17,
            "rows": 1, "scenario": scenario, "checksum": 0, "instrumented": True,
            "micro": [
                {"name": "croquis.oxc.original.bytes", "calls": 4, "units": 400},
                {"name": "croquis.oxc.recovery.bytes", "calls": recovery, "units": 100 * recovery},
            ],
            "allocation": {"calls": calls, "requestedBytes": allocated},
        }
    irreparable = (fixture("irreparable", 10, 1000), fixture("irreparable", 6, 600))
    assert profile_pair(*irreparable)["callsSaved"] == 4
    for scenario in ("valid", "repairable"):
        row = fixture(scenario, 10, 1000)
        assert profile_pair(row, copy.deepcopy(row))["callsSaved"] == 0
    cases = []
    def case(name, pair, mutate):
        value = copy.deepcopy(pair)
        mutate(value)
        cases.append((name, value))
    case("original-call-mismatch", irreparable, lambda value: value[1]["micro"][0].__setitem__("calls", 3))
    case("repair-not-observed", (fixture("repairable", 10, 1000), fixture("repairable", 10, 1000)), lambda value: value[1]["micro"].pop())
    case("no-removed-copy", irreparable, lambda value: value[1]["allocation"].__setitem__("calls", 10))
    case("two-removed-copies", irreparable, lambda value: value[1]["allocation"].__setitem__("calls", 2))
    case("too-few-removed-bytes", irreparable, lambda value: value[1]["allocation"].__setitem__("requestedBytes", 999))
    case("valid-allocation-changed", (fixture("valid", 10, 1000), fixture("valid", 10, 1000)), lambda value: value[1]["allocation"].__setitem__("calls", 9))
    case("source-mismatch", irreparable, lambda value: value[1].__setitem__("sourceHash", 18))
    case("normal-sample-as-profile", irreparable, lambda value: value[1].__setitem__("instrumented", False))
    case("zero-iterations", irreparable, lambda value: [row.__setitem__("iterations", 0) for row in value])
    case("unsupported-scenario", irreparable, lambda value: [row.__setitem__("scenario", "unknown") for row in value])
    rejected = []
    for name, pair in cases:
        try:
            profile_pair(*pair)
        except (AssertionError, KeyError, TypeError, ValueError):
            rejected.append(name)
        else:
            raise AssertionError("invalid clone comparison admitted: " + name)
    print("clone-contract-controls=" + json.dumps({"positive": 3, "rejected": rejected}))


def main():
    mode = sys.argv[1]
    if mode == "--self-test":
        self_test()
        return
    root = Path(sys.argv[2])
    assert mode in ("tiny", "matrix")
    if mode == "tiny":
        profile_identity(root)
    sizes = [4096] if mode == "tiny" else [128 * 1024, 1024 * 1024, 10 * 1024 * 1024]
    rounds = [1] if mode == "tiny" else [1, 2, 3]
    summary = {
        "scope": "bounded repeated-source parse+analysis; generation/report outside inner window, no backend or project cold-time claim",
        "timingMetric": "normal inner wall time, separate from instrumented allocations",
        "rssScope": "whole process includes source generation; /usr/bin/time raw receipts only",
        "cases": [],
    }
    for scenario in ("irreparable", "valid", "repairable"):
        for size in sizes:
            normal = {
                role: [sample(root / f"normal-{scenario}-{role}-{size}-{n}.json") for n in rounds]
                for role in ("base", "head")
            }
            assert len({signature(row) for rows in normal.values() for row in rows}) == 1
            assert all(not row["instrumented"] for rows in normal.values() for row in rows)
            profiles = {
                role: sample(root / f"profile-{scenario}-{role}-{size}.json")
                for role in ("base", "head")
            }
            # Instrumented repetition count may differ; the actual source must match.
            for row in profiles.values():
                for key in ("sourceBytes", "sourceHash", "rows", "scenario"):
                    assert row[key] == normal["base"][0][key]
            summary["cases"].append({
                "scenario": scenario, "minimumSourceBytes": size,
                "sourceBytes": normal["base"][0]["sourceBytes"],
                "sourceHash": normal["base"][0]["sourceHash"],
                "normal": {
                    role: {
                        "samplesNs": [row["elapsedNs"] for row in rows],
                        "medianNs": statistics.median(row["elapsedNs"] for row in rows),
                        "iterations": rows[0]["iterations"],
                    }
                    for role, rows in normal.items()
                },
                "allocation": profile_pair(profiles["base"], profiles["head"]),
            })
    (root / f"clone-{mode}-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print("cloneComparison=" + json.dumps({"mode": mode, "comparison": summary}, separators=(",", ":")))


if __name__ == "__main__":
    main()
