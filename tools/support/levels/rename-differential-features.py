#!/usr/bin/env python3
"""Replay the #6832 differential feature rename on the current checkout."""

import argparse
import re
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
# Published v0.429.1 Cargo feature names are retained only as manifest aliases
# for the support-policy deprecation window. Active selectors still use legacy-*.
PUBLISHED_ALIAS_MANIFESTS = {
    "crates/vize_atelier_core/Cargo.toml",
    "crates/vize_atelier_dom/Cargo.toml",
    "crates/vize_atelier_jsx/Cargo.toml",
    "crates/vize_atelier_sfc/Cargo.toml",
    "crates/vize_atelier_ssr/Cargo.toml",
    "crates/vize_atelier_vapor/Cargo.toml",
    "crates/vize_canon/Cargo.toml",
    "crates/vize_croquis/Cargo.toml",
    "crates/vize_l1/Cargo.toml",
    "crates/vize_l1_to_l2/Cargo.toml",
    "crates/vize_patina/Cargo.toml",
}


def eligible(path: Path) -> bool:
    if path.as_posix() in EXTRA_PATHS:
        return True
    parts = path.parts
    if path.suffix not in SUFFIXES or "fixtures" in parts or "snapshots" in parts:
        return False
    if parts[0] == "tests":
        return len(parts) > 1 and parts[1] in {"tooling", "davinci_test_support"}
    return parts[0] in {".github", "crates"}


def rewrite(source: str, relative: Path) -> str:
    protected_aliases = []
    manifest_path = relative.as_posix()
    if manifest_path in PUBLISHED_ALIAS_MANIFESTS:
        for old, new in RENAMES:
            if (
                old == "davinci-dom-differential"
                and manifest_path != "crates/vize_atelier_sfc/Cargo.toml"
            ):
                continue
            alias = f'{old} = ["{new}"]'
            token = f"__VIZE_PUBLISHED_FEATURE_ALIAS_{len(protected_aliases)}__"
            source, count = re.subn(rf"(?m)^{re.escape(alias)}$", token, source)
            if count:
                protected_aliases.append((token, alias))
    for index, message in enumerate(PROTECTED_MESSAGES):
        source = source.replace(message, f"__VIZE_PROTECTED_MESSAGE_{index}__")
    for old, new in RENAMES:
        source = source.replace(old, new)
    for index, message in enumerate(PROTECTED_MESSAGES):
        source = source.replace(f"__VIZE_PROTECTED_MESSAGE_{index}__", message)
    for token, alias in protected_aliases:
        source = source.replace(token, alias)
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
        rewritten = rewrite(source, relative)
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
