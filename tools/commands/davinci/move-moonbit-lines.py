#!/usr/bin/env python3
"""Replay #6841's MoonBit position-index move on fresh main."""

from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
old = ROOT / "crates/vize_dialect_moonbit/src/lines.rs"
new = ROOT / "davinci/vize_l1/src/lang/moonbit/lines.rs"

if old.is_file() and not new.exists():
    new.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(["git", "mv", str(old), str(new)], cwd=ROOT, check=True)
elif new.is_file() and (
    not old.exists()
    or "pub use vize_l1::lang::moonbit::lines" in old.read_text()
):
    pass
else:
    raise SystemExit("expected the original implementation or its L1 re-export")
