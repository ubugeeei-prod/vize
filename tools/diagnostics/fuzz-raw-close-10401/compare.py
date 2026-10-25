"""Require complete packet parity, retaining measurements without new budgets."""

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def load(root, name, phase):
    return json.loads((root / name / f"{phase}.stdout").read_text())


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--manifest", type=Path, default=ROOT / "manifest.json")
    parser.add_argument("--pins", type=Path, default=ROOT / "pins.json")
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    original = load(args.baseline, "original", "controls")
    assert len(original) == 91
    pins = json.loads(args.pins.read_text())
    for root in [args.baseline, args.candidate]:
        original_bytes = (root / "original/controls.stdout").read_bytes()
        assert hashlib.sha256(original_bytes).hexdigest() == pins["original_91_packets_sha256"]
    summary = []
    for case in manifest["cases"]:
        name = case["id"]
        before = load(args.baseline, name, "controls")
        after = load(args.candidate, name, "controls")
        assert len(before) == len(after) == 91, name
        assert before == after, f"full AST/errors/spans changed: {name}"
        assert before[:90] == original[:90], name
        if case["source"] == "append-only counterexample, not original artifact":
            assert before[-1] == original[-1], f"appended tail changed original failure packet: {name}"
        inspect = load(args.baseline, name, "inspect")
        assert inspect == load(args.candidate, name, "inspect"), name
        row = {"id": name, "input_sha256": case["sha256"], "inspect": inspect}
        program_before = load(args.baseline, name, "program-observer")
        program_after = load(args.candidate, name, "program-observer")
        assert program_before == program_after, f"program/comments/tokens/errors differ: {name}"
        row["program_observer"] = {
            "panicked": program_after["panicked"],
            "token_kind_counts": program_after["token_kind_counts"],
            "raw_gt_ownership": program_after["raw_gt_ownership"],
        }
        if case["suffix_utf8"]:
            suffix_before = load(args.baseline, name, "suffix-observer")
            suffix_after = load(args.candidate, name, "suffix-observer")
            assert suffix_before == suffix_after, f"standalone suffix differs: {name}"
            row["standalone_suffix_observer"] = {
                "panicked": suffix_after["panicked"],
                "raw_gt_ownership": suffix_after["raw_gt_ownership"],
            }
        for phase in ["raw", "guarded"]:
            measures = [load(directory, name, phase) for directory in [args.baseline, args.candidate]]
            observations = [
                {key: value for key, value in measurement.items() if key != "elapsed_ns"}
                for measurement in measures
            ]
            assert observations[0] == observations[1], f"admission/result changed: {name}/{phase}"
            row[phase] = dict(zip(["baseline", "candidate"], measures))
        counters = [root / name / "raw-counters.json" for root in [args.baseline, args.candidate]]
        if all(path.exists() for path in counters):
            row["callgrind"] = dict(zip(["baseline", "candidate"], [json.loads(path.read_text()) for path in counters]))
        summary.append(row)
    print(json.dumps({"full_packet_parity": True, "cases": summary}, indent=2))


if __name__ == "__main__":
    main()
