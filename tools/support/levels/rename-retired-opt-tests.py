#!/usr/bin/env python3
"""Replay the #6832 host-retirement test path moves before updating contents."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MOVES = {
    "davinci/vize_davinci/tests/davinci_opt_cli.rs": "davinci/vize_davinci/tests/dump_croquis.rs",
    "davinci/vize_davinci/tests/davinci_opt_dumps.rs": "davinci/vize_davinci/tests/dump_collector.rs",
    "davinci/vize_davinci/tests/davinci_opt_remarks.rs": "davinci/vize_davinci/tests/dump_remarks.rs",
    "davinci/vize_davinci/tests/spolvero_feed.rs": "davinci/vize_davinci/tests/stage_feed.rs",
}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=("moves", "check"))
    args = parser.parse_args()

    missing = []
    for old, new in MOVES.items():
        source, target = ROOT / old, ROOT / new
        if args.phase == "moves" and source.exists() and not target.exists():
            subprocess.run(["git", "mv", old, new], cwd=ROOT, check=True)
        elif not target.exists():
            missing.append(new)
        if args.phase == "check" and source.exists():
            missing.append(old)
    if missing:
        raise SystemExit("test path move incomplete:\n" + "\n".join(missing))


if __name__ == "__main__":
    main()
