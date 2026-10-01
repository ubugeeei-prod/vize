#!/usr/bin/env python3
"""Replay the #6833 diagnostic renderer move into its CLI owner."""
import argparse
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
OLD = Path("davinci/vize_davinci")
NEW = Path("crates/vize")
MOVES = {
    OLD / "src/render.rs": NEW / "src/render.rs",
    OLD / "src/render": NEW / "src/render",
    OLD / "tests/diagnostic_render.rs": NEW / "tests/diagnostic_render.rs",
    OLD / "tests/diagnostic_render": NEW / "tests/diagnostic_render",
    OLD / "tests/render_why.rs": NEW / "tests/render_why.rs",
    OLD / "tests/snapshots/diagnostic_render": NEW / "tests/snapshots/diagnostic_render",
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
            raise SystemExit("Missing moved renderer: " + str(new))
    if phase != "integrate":
        return
    owned_sources = [ROOT / NEW / "src/render.rs"] + list((ROOT / NEW / "src/render").rglob("*.rs"))
    for p in owned_sources:
        s = p.read_text().replace("crate::diagnostic", "vize_l0::diag").replace("vize_carton::i18n", "vize_l0::i18n")
        p.write_text(s.replace("crate::fact", "vize_l0::fact").replace("crate::pass", "vize_l0::pass"))
    for p in (ROOT / NEW / "src/commands").rglob("*.rs"):
        s = p.read_text()
        updated = s.replace("vize_davinci::render", "crate::render")
        if updated != s:
            p.write_text(updated)
    tests = [ROOT / NEW / "tests/diagnostic_render.rs", ROOT / NEW / "tests/render_why.rs"]
    tests += list((ROOT / NEW / "tests/diagnostic_render").rglob("*.rs"))
    for p in tests:
        s = p.read_text().replace("vize_davinci::render", "vize::render")
        s = s.replace("vize_davinci::diagnostic", "vize_l0::diag")
        s = s.replace("vize_davinci::fact", "vize_l0::fact").replace("vize_davinci::pass", "vize_l0::pass")
        s = s.replace("vize_davinci --test diagnostic_render", "vize --test diagnostic_render")
        s = s.replace('Exemption::new("vize_davinci", "render-why-fixture")', 'Exemption::new("vize", "render-why-fixture")')
        p.write_text(s)
    p = ROOT / OLD / "src/lib.rs"
    s = p.read_text().replace("pub mod render;\n", "")
    p.write_text(re.sub(r"//! - \[`render`\].*\n//!   surface shares.*\n", "", s))
    p = ROOT / NEW / "src/lib.rs"
    s = p.read_text()
    if "pub mod render;" not in s:
        s = s.replace("mod commands;", "extern crate alloc;\n\npub mod render;\n\nmod commands;")
        p.write_text(s)
    p = ROOT / OLD / "Cargo.toml"
    s = p.read_text().replace("# Terminal display widths for the diagnostic renderer (no_std).\nunicode-width = { workspace = true }\n", "")
    p.write_text(s)
    p = ROOT / NEW / "Cargo.toml"
    s = p.read_text()
    if "unicode-width" not in s:
        p.write_text(s.replace("[dependencies]\n", "[dependencies]\nunicode-width.workspace = true\n"))
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
    # CLI rendering is outside native level storage; retain its byte fixtures.
    rows = [row for row in p.read_text().splitlines() if str(OLD / "src/render") not in row]
    p.write_text("\n".join(rows) + "\n")
    p = ROOT / "tests/tooling/davinci-diagnostic-catalog.test.ts"
    s = p.read_text().replace("-p vize_davinci --test diagnostic_render", "-p vize --test diagnostic_render")
    p.write_text(re.sub(r'"davinci",(\s*)"vize_davinci",', r'"crates",\1"vize",', s))
    p = ROOT / "docs/davinci/plan/test-suites.md"
    p.write_text(p.read_text().replace("-p vize_davinci --test diagnostic_render", "-p vize --test diagnostic_render"))


if __name__ == "__main__":
    main()
