"""Out-of-tree audit runner; never edits parser source or the original input.

Uses the unchanged 7bd295 diagnostic CLI. Binary provenance must be established
by the caller's fresh exact-source build; published NPM parsers are not accepted.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

SOURCES = {
    "baseline": "3a4bf713f85589a50da5d0a7aaf8ae33fdcdc004",
    "candidate": "7bd29589171cdaae96aacca6ee7b213ea6dd347b",
}
ROOT = Path(__file__).resolve().parent


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--observer", type=Path, required=True)
    parser.add_argument("--role", choices=SOURCES, required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, default=ROOT / "manifest.json")
    parser.add_argument("--callgrind", action="store_true")
    args = parser.parse_args()
    assert args.source_sha == SOURCES[args.role]
    binary = args.binary.resolve(strict=True)
    observer = args.observer.resolve(strict=True)
    assert not args.out.exists(), "fresh output directory required"
    args.out.mkdir(parents=True)
    manifest = json.loads(args.manifest.read_text())
    metadata = {
        "role": args.role,
        "source_sha": args.source_sha,
        "binary": str(binary),
        "binary_sha256": sha256(binary),
        "observer": str(observer),
        "observer_sha256": sha256(observer),
        "manifest_sha256": sha256(args.manifest),
        "original_sha256": manifest["original_sha256"],
        "cases": [],
    }
    failed = False
    for case in manifest["cases"]:
        source = args.manifest.parent / "cases" / f"{case['id']}.input"
        assert sha256(source) == case["sha256"]
        target = args.out / case["id"]
        target.mkdir()
        record = {"id": case["id"], "input_sha256": case["sha256"], "commands": []}
        for phase in ["inspect", "controls", "raw", "guarded"]:
            command = [str(binary), phase, str(source)]
            if phase in ("raw", "guarded"):
                command.append("1")
            result = run(command, target / phase, timeout=90)
            record["commands"].append(result)
            failed |= result["status"] != 0
        result = run([str(observer), str(source)], target / "program-observer", timeout=90)
        record["commands"].append(result)
        failed |= result["status"] != 0
        if case["suffix_utf8"]:
            suffix = target / "standalone-suffix.ts"
            suffix.write_text(case["suffix_utf8"])
            result = run([str(observer), str(suffix)], target / "suffix-observer", timeout=90)
            record["commands"].append(result)
            failed |= result["status"] != 0
        if args.callgrind and case["id"] in {
            "original", "line-comment-control", "line-comment-greater",
            "double-string-control", "double-string-greater",
            "binary-token-control", "binary-token-greater",
            "prefix-4096-control", "prefix-4096-greater",
            "prefix-8192-control", "prefix-8192-greater",
            "prefix-16384-control", "prefix-16384-greater",
        }:
            command = [
                "valgrind", "--tool=callgrind",
                f"--callgrind-out-file={target / 'raw.callgrind'}",
                str(binary), "raw", str(source), "1",
            ]
            result = run(command, target / "raw-callgrind", timeout=120)
            record["commands"].append(result)
            failed |= result["status"] != 0
            profile = target / "raw.callgrind"
            if profile.exists():
                with (target / "raw-source-profile.txt").open("wb") as out:
                    subprocess.run(["callgrind_annotate", "--inclusive=no", "--tree=both", "--auto=yes", str(profile)], stdout=out, check=True)
                counter = Path(__file__).with_name("callgrind_counters.py")
                with (target / "raw-counters.json").open("wb") as out:
                    subprocess.run([sys.executable, str(counter), str(profile)], stdout=out, check=True)
        metadata["cases"].append(record)
        (args.out / "receipt.json").write_text(json.dumps(metadata, indent=2) + "\n")
    return int(failed)


def run(command, prefix, timeout):
    record = {"argv": command, "timeout_seconds": timeout}
    with prefix.with_suffix(".stdout").open("wb") as stdout:
        with prefix.with_suffix(".stderr").open("wb") as stderr:
            try:
                result = subprocess.run(command, stdout=stdout, stderr=stderr, timeout=timeout)
                record["status"] = result.returncode
            except subprocess.TimeoutExpired:
                record["status"] = "timeout"
    prefix.with_suffix(".command.json").write_text(json.dumps(record, indent=2) + "\n")
    return record


if __name__ == "__main__":
    sys.exit(main())
