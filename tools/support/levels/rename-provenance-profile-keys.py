#!/usr/bin/env python3
"""Replay the bounded L1→L2 provenance profile key rename on current main."""

import argparse
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
TARGET = ROOT / "crates/vize_l1_to_l2/src/lower/benchmark.rs"
SUFFIXES = (
    "record",
    "records",
    "vector_capacity_bytes",
    "record_bytes",
    "rule_heap_strings",
    "rule_heap_capacity_bytes",
    "before_heap_strings",
    "before_heap_capacity_bytes",
    "after_heap_strings",
    "after_heap_capacity_bytes",
)


def rewrite(source: str) -> tuple[str, int]:
    changed = 0
    for suffix in SUFFIXES:
        old = f'"davinci.lower.provenance.{suffix}"'
        new = f'"l1_to_l2.provenance.{suffix}"'
        old_count = source.count(old)
        new_count = source.count(new)
        if old_count + new_count != 1:
            raise ValueError(f"expected exactly one old or new key: {suffix}")
        if old_count:
            source = source.replace(old, new)
            changed += 1
    return source, changed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="rewrite the one source file")
    mode.add_argument("--check", action="store_true", help="require all ten current keys")
    args = parser.parse_args()

    source = TARGET.read_text(encoding="utf-8")
    rewritten, changed = rewrite(source)
    if args.write and changed:
        TARGET.write_text(rewritten, encoding="utf-8")
    print(f"{changed} provenance profile keys {'rewritten' if args.write else 'still old'}")
    return int(args.check and changed != 0)


if __name__ == "__main__":
    raise SystemExit(main())
