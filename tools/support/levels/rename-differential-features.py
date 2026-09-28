#!/usr/bin/env python3
"""Replay the #6832 differential feature rename on the current checkout."""

import argparse
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
SUFFIXES = {".rs", ".toml", ".ts", ".mjs", ".yml", ".yaml"}
EXTRA_PATHS = (
    "tools/commands/fixtures/davinci-dom-corpus-workflow.rs",
    "tools/support/compat/fixtures/davinci-dom-corpus-workflow.mjs",
    "docs/davinci/plan/test-suites.md",
)
RENAMES = (
    ("davinci-dom-differential", "legacy-dom-differential"),
    ("davinci-differential", "legacy-differential"),
)
PROTECTED_MESSAGES = (
    "davinci-differential (P1-",
    "davinci-differential corpus ",
    "davinci-differential totals:",
    "vize_s1 --features davinci-differential",
)


def eligible(path: Path) -> bool:
    if path.as_posix() in EXTRA_PATHS:
        return True
    parts = path.parts
    if path.suffix not in SUFFIXES or "fixtures" in parts or "snapshots" in parts:
        return False
    if parts[0] == "tests":
        return len(parts) > 1 and parts[1] in {"tooling", "davinci_test_support"}
    return parts[0] in {".github", "crates"}


def rewrite(source: str) -> str:
    for index, message in enumerate(PROTECTED_MESSAGES):
        source = source.replace(message, f"__VIZE_PROTECTED_MESSAGE_{index}__")
    for old, new in RENAMES:
        source = source.replace(old, new)
    for index, message in enumerate(PROTECTED_MESSAGES):
        source = source.replace(f"__VIZE_PROTECTED_MESSAGE_{index}__", message)
    return source


def tracked_paths() -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "-z", "--", ".github", "crates", "tests", *EXTRA_PATHS],
        cwd=ROOT,
        check=True,
        capture_output=True,
    )
    return [
        Path(raw.decode())
        for raw in result.stdout.split(b"\0")
        if raw and eligible(Path(raw.decode()))
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="rewrite tracked source files")
    mode.add_argument("--check", action="store_true", help="fail if old selectors remain")
    args = parser.parse_args()

    changed = []
    for relative in tracked_paths():
        path = ROOT / relative
        source = path.read_text(encoding="utf-8")
        rewritten = rewrite(source)
        if rewritten == source:
            continue
        changed.append(str(relative))
        if args.write:
            path.write_text(rewritten, encoding="utf-8")

    for path in changed:
        print(path)
    if args.check and changed:
        return 1
    print(f"{len(changed)} files {'rewritten' if args.write else 'contain old selectors'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
