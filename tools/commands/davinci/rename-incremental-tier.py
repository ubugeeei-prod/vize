#!/usr/bin/env python3
"""Replay #7304's incremental crate rename without changing runtime behavior.

Run --phase moves, commit the byte-exact moves, then run --phase references.
Archived key-capture logs retain their original build/package identities.
"""

import argparse
from pathlib import Path
import subprocess

OLD = "vize_resident"
NEW = "vize_incremental"
PRESERVED = {
    "davinci/vize_l0/tests/fixtures/keys/capture-v2/old-build.stdout",
    "davinci/vize_l0/tests/fixtures/keys/capture-v2/old-build.stderr",
    "tools/commands/davinci/rename-incremental-tier.py",
    "docs/davinci/decisions/2026-10-03-incremental-tier-name.md",
}


def move(root: Path, old: str, new: str) -> None:
    source, destination = root / old, root / new
    if source.exists() and not destination.exists():
        subprocess.run(["git", "mv", old, new], cwd=root, check=True)
    elif source.exists() == destination.exists():
        raise SystemExit(f"expected exactly one rename endpoint: {old}, {new}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--phase", choices=("moves", "references", "all"), default="all")
    args = parser.parse_args()
    root = args.root.resolve()
    if args.phase in ("moves", "all"):
        move(root, f"crates/{OLD}", f"crates/{NEW}")
        move(
            root,
            f"docs/davinci/plan/croquis-consumption/{OLD}.md",
            f"docs/davinci/plan/croquis-consumption/{NEW}.md",
        )
    if args.phase in ("references", "all"):
        paths = subprocess.check_output(["git", "ls-files", "-z"], cwd=root)
        for encoded in paths.split(b"\0"):
            if not encoded:
                continue
            relative = encoded.decode()
            path = root / relative
            if relative in PRESERVED or not path.is_file():
                continue
            content = path.read_bytes()
            if OLD.encode() in content:
                path.write_bytes(content.replace(OLD.encode(), NEW.encode()))


if __name__ == "__main__":
    main()
