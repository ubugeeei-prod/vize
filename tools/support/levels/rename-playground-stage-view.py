#!/usr/bin/env python3
"""Replay the playground stage-view file and source renames for #6832."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MOVES = {
    "playground/src/features/stages/DavinciPlayground.css": "playground/src/features/stages/StagePlayground.css",
    "playground/src/features/stages/DavinciPlayground.vue": "playground/src/features/stages/StagePlayground.vue",
    "playground/src/features/stages/FolioView.css": "playground/src/features/stages/DumpView.css",
    "playground/src/features/stages/FolioView.vue": "playground/src/features/stages/DumpView.vue",
    "playground/src/features/stages/FolioDiffView.vue": "playground/src/features/stages/DumpDiffView.vue",
    "playground/src/features/stages/folioLines.ts": "playground/src/features/stages/dumpLines.ts",
    "playground/src/features/stages/folioLines.test.ts": "playground/src/features/stages/dumpLines.test.ts",
    "playground/src/features/stages/useDavinciLadder.ts": "playground/src/features/stages/useStageLadder.ts",
    "playground/e2e/davinci-ladder.test.ts": "playground/e2e/stage-ladder.test.ts",
}
REPLACEMENTS = {
    "DavinciPlayground": "StagePlayground",
    "useDavinciLadder": "useStageLadder",
    "FolioDiffView": "DumpDiffView",
    "FolioView": "DumpView",
    "FolioLine": "DumpLine",
    "folioLines": "dumpLines",
    "folioTokens": "dumpTokens",
    "FOLIO_TOKEN": "DUMP_TOKEN",
    "FOLIO_GROUPS": "DUMP_GROUPS",
}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=("moves", "references", "check"))
    args = parser.parse_args()

    if args.phase == "moves":
        for old, new in MOVES.items():
            source, target = ROOT / old, ROOT / new
            if source.exists() and not target.exists():
                subprocess.run(["git", "mv", old, new], cwd=ROOT, check=True)
            elif not target.exists():
                raise SystemExit(f"missing rename input: {old}")
        return

    stale = []
    paths = [
        path
        for base in (ROOT / "playground/src", ROOT / "playground/e2e")
        for path in base.rglob("*")
        if path.is_file() and path.suffix in {".ts", ".vue", ".css", ".snap"}
    ]
    paths.append(ROOT / "docs/davinci/plan/v-on-corpus/playground--src.tsv")
    for path in paths:
        content = path.read_text()
        updated = content
        for old, new in REPLACEMENTS.items():
            updated = updated.replace(old, new)
        if path.name == "stage-ladder.test.ts":
            updated = updated.replace(
                'describe("Davinci stage ladder from the real compiler"',
                'describe("Stage ladder from the real compiler"',
            )
        if args.phase == "references" and updated != content:
            path.write_text(updated)
        if args.phase == "check" and updated != content:
            stale.append(str(path.relative_to(ROOT)))

    if args.phase == "check":
        stale.extend(old for old in MOVES if (ROOT / old).exists())
        if stale:
            raise SystemExit("stale playground names:\n" + "\n".join(sorted(set(stale))))


if __name__ == "__main__":
    main()
