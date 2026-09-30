#!/usr/bin/env python3
"""Replay the Rust guest SDK crate rename on a fresh main checkout.

Keep archived 0.1.2 guest sources and the released WIT bytes fixed. The JS/TS
extension-sdk package is a separate product and retains its package name.
"""

from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
OLD = "vize_extension_sdk"
NEW = "vize_guest"
OLD_DIR = ROOT / "crates" / OLD
NEW_DIR = ROOT / "crates" / NEW
ARCHIVED = (
    "davinci/vize_extension_host/tests/fixtures/sdk-0.1.2/",
    "davinci/vize_extension_host/tests/guests/sdk-hello-0-1-2/",
    "davinci/vize_extension_host/tests/guests/expression-echo-0-1-2/",
)


def git(*args: str) -> bytes:
    return subprocess.check_output(["git", *args], cwd=ROOT)


if OLD_DIR.exists() and not NEW_DIR.exists():
    subprocess.run(["git", "mv", str(OLD_DIR), str(NEW_DIR)], cwd=ROOT, check=True)
elif OLD_DIR.exists() == NEW_DIR.exists():
    raise SystemExit("expected exactly one guest SDK directory")

old_ledger = ROOT / "docs/davinci/plan/croquis-consumption/vize_extension_sdk.md"
new_ledger = ROOT / "docs/davinci/plan/croquis-consumption/vize_guest.md"
if old_ledger.exists() and not new_ledger.exists():
    subprocess.run(["git", "mv", str(old_ledger), str(new_ledger)], cwd=ROOT, check=True)

for encoded in git("ls-files", "-z").split(b"\0"):
    if not encoded:
        continue
    relative = encoded.decode()
    if (
        relative.startswith(ARCHIVED)
        or relative.startswith("davinci/vize_guest/wit/")
        or relative == "tools/commands/davinci/rename-guest-sdk.py"
        or relative == "docs/davinci/decisions/2026-09-28-guest-sdk-crate-axis.md"
    ):
        continue
    path = ROOT / relative
    if not path.is_file():
        continue
    content = path.read_bytes()
    if OLD.encode() not in content:
        continue
    updated = content.replace(OLD.encode(), NEW.encode())
    if relative == "tests/tooling/davinci-extension-sdk.test.ts":
        # This assertion intentionally names the frozen historical package.
        updated = updated.replace(
            b'cargo.match(/^vize_guest = .*$/mu)?.slice(), [\n      \'vize_guest = { path = "../../fixtures/sdk-0.1.2" }\'',
            b'cargo.match(/^vize_extension_sdk = .*$/mu)?.slice(), [\n      \'vize_extension_sdk = { path = "../../fixtures/sdk-0.1.2" }\'',
        )
    path.write_bytes(updated)
