#!/usr/bin/env python3
"""Replay the #6833 summary ownership move without changing serialized bytes."""
import argparse
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
OLD = Path("davinci/vize_davinci")
NEW = Path("davinci/vize_l2")
MOVES = {
    OLD / "src/summary.rs": NEW / "src/summary.rs",
    OLD / "src/summary": NEW / "src/summary",
    OLD / "tests/sfc_summary": NEW / "tests/sfc_summary",
    OLD / "tests/global_summary.rs": NEW / "tests/global_summary.rs",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=("moves", "integrate", "check"))
    phase = parser.parse_args().phase
    for old, new in MOVES.items():
        source, target = ROOT / old, ROOT / new
        if phase == "moves" and source.exists() and not target.exists():
            target.parent.mkdir(parents=True, exist_ok=True)
            subprocess.run(["git", "mv", str(old), str(new)], cwd=ROOT, check=True)
        elif not target.exists():
            raise SystemExit("Missing moved summary: " + str(new))
    if phase != "integrate":
        return
    for base in (ROOT / "crates", ROOT / "davinci"):
        for p in base.rglob("*.rs"):
            s = p.read_text()
            updated = s.replace("vize_davinci::summary", "vize_l2::summary")
            if p.is_relative_to(ROOT / NEW / "src/summary") or p == ROOT / NEW / "src/summary.rs":
                updated = updated.replace("crate::dump", "vize_l0::dump").replace("crate::fact", "vize_l0::fact")
            if p.is_relative_to(ROOT / NEW / "tests/sfc_summary") or p == ROOT / NEW / "tests/global_summary.rs":
                updated = updated.replace("-p vize_davinci", "-p vize_l2").replace("vize_davinci::dump", "vize_l0::dump")
            if updated != s:
                p.write_text(updated)
    p = ROOT / OLD / "src/lib.rs"
    s = p.read_text().replace("pub mod summary;\n", "")
    s = re.sub(r"//! - \[`summary`\].*\n//!   fingerprinted per declaration.*\n", "", s)
    p.write_text(s)
    p = ROOT / NEW / "src/lib.rs"
    s = p.read_text()
    if "pub mod summary;" not in s:
        p.write_text(s.replace("pub mod scope;", "pub mod scope;\npub mod summary;"))
    for name in ("vize_croquis", "vize_maestro"):
        p = ROOT / "crates" / name / "Cargo.toml"
        s = p.read_text()
        if "vize_l2.workspace" not in s:
            p.write_text(s.replace("vize_davinci.workspace = true", "vize_davinci.workspace = true\nvize_l2.workspace = true"))
    for p in (ROOT / "docs").rglob("*.md"):
        s = p.read_text()
        def link(match):
            value = match.group(0)
            if value.startswith(("https:", "http:")):
                return value
            for old, new in MOVES.items():
                value = value.replace(str(old), str(new))
            return value
        updated = re.sub(r"(?<=\]\()[^)]*", link, s)
        if updated != s:
            p.write_text(updated)
    p = ROOT / "docs/davinci/plan/storage-inventory.tsv"
    s = p.read_text().replace(str(OLD / "src/summary"), str(NEW / "src/summary"))
    rows = []
    for row in s.splitlines():
        fields = row.split("\t")
        if len(fields) > 2 and fields[2].startswith(str(NEW / "src/summary")):
            fields[0] = "l2"
        rows.append("\t".join(fields))
    p.write_text("\n".join(rows) + "\n")


if __name__ == "__main__":
    main()
