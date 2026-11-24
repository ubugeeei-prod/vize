from pathlib import Path
import argparse
import subprocess
import tempfile

MUTATIONS = {
    "left": ("p < start", "p <= start", 1),
    "rank-before": ("return length..length", "return 0..0", 1),
    "short-end": ("usize::from(point <= last)", "usize::from(point < last)", 1),
    "upper": ("tail.partition_point(|&point| point <= last)", "tail.partition_point(|&point| point < last)", 1),
    "prefix": ("2 + usize::from(*third <= last)", "2 + usize::from(*third < last)", 1),
    "tail-offset": ("4 + tail.partition_point", "3 + tail.partition_point", 1),
    "empty": ("end - u32::from(start < end)", "end.saturating_sub(1)", 2),
    "single-lower": ("start <= cursor", "start < cursor", 1),
    "single-insertion": ("(start == end && cursor == start)", "false", 1),
    "single-rank": ("cursor < start", "cursor <= start", 1),
}


def main():
    parser = argparse.ArgumentParser(description="Replay the positive recovery-window proof and each isolated negative control with Kani 0.68.0.")
    parser.add_argument("checkout", type=Path, help="Checkout of the recovery cursor candidate")
    parser.add_argument("--check-inputs", action="store_true", help="Verify every mutation matches its intended source without running Kani")
    args = parser.parse_args()
    source = args.checkout / "crates/vize_incremental/providers/graph/src/recovery/positions"
    kernel = (source / "window.rs").read_text()
    harness = (source / "kani.rs").read_bytes()
    cases = {"original": kernel}
    for name, (before, after, count) in MUTATIONS.items():
        if kernel.count(before) != count:
            raise SystemExit(f"{name} expected {count} occurrences")
        cases[name] = kernel.replace(before, after)
    if args.check_inputs:
        print(f"{len(MUTATIONS)} anchored controls matched the source.")
        return
    with tempfile.TemporaryDirectory() as directory:
        for name, candidate in cases.items():
            case = Path(directory) / name
            case.mkdir()
            (case / "window.rs").write_text(candidate)
            (case / "kani.rs").write_bytes(harness)
            result = subprocess.run(["kani", "kani.rs", "--output-format=terse"], cwd=case, capture_output=True, text=True)
            log = result.stdout + result.stderr
            print(name, log, flush=True)
            positive = name == "original"
            marker = "VERIFICATION:- SUCCESSFUL" if positive else "VERIFICATION:- FAILED"
            if marker not in log or "window_matches_each_requested_cursor" not in log or (result.returncode == 0) != positive:
                raise SystemExit(f"Unexpected proof result for {name}")
            if positive and "6 of 6 cover properties satisfied" not in log:
                raise SystemExit("The positive harness did not reach all six covers")


if __name__ == "__main__":
    main()
