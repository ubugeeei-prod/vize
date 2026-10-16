#!/usr/bin/env python3
"""Verify only the imported checked Corsa graph, its licenses and source census."""
import csv
import hashlib
from pathlib import Path
import sys

PACKAGES = {"corsa", "corsa_core", "corsa_jsonrpc", "corsa_client", "corsa_lsp", "corsa_orchestrator"}


def verify(root):
    records = list(csv.DictReader((root / "vendor/corsa-source.tsv").open(), delimiter="\t"))
    expected = set()
    for record in records:
        package, relative = record["package"], Path(record["path"])
        if package not in PACKAGES or relative.is_absolute() or ".." in relative.parts:
            raise ValueError("unexpected package/source path")
        path = root / "vendor" / package / relative
        if path in expected or path.is_symlink() or not path.is_file():
            raise ValueError(f"missing, duplicate or replaced source: {path}")
        expected.add(path)
        body = path.read_bytes()
        if len(body) != int(record["bytes"]) or hashlib.sha256(body).hexdigest() != record["sha256"]:
            raise ValueError(f"imported source changed: {path}")
    actual = {path for package in PACKAGES for path in (root / "vendor" / package).rglob("*") if path.is_file() or path.is_symlink()}
    if actual != expected:
        raise ValueError(f"source census differs: missing={expected-actual}, extra={actual-expected}")
    licenses = [(root / "vendor" / package / "LICENSE").read_bytes() for package in PACKAGES]
    if len(set(licenses)) != 1 or b"MIT License" not in licenses[0]:
        raise ValueError("all imported packages must retain the same complete upstream MIT notice")
    print(f"Verified {len(records)} checked Corsa source members and six complete MIT notices.")


if __name__ == "__main__":
    verify(Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path.cwd())
