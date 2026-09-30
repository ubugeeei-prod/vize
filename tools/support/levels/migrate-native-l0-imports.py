#!/usr/bin/env python3
"""Replay native consumer migration to the canonical L0 runtime (#6833)."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[3]
MODULES = {'diagnostic': 'diag', 'witness': 'diag::verify', **{
    name: name for name in ('id', 'side_table', 'key', 'pass', 'fact', 'stage', 'dump', 'assert_dump_snapshot')
}}
for crate in (ROOT / 'davinci').iterdir():
    if not crate.is_dir() or crate.name == 'vize_davinci':
        continue
    for source in crate.rglob('*.rs'):
        if 'fixtures' in source.parts or 'versions' in source.parts:
            continue
        original = source.read_text()
        changed = re.sub(r'vize_davinci::([a-z_]+)\b',
                         lambda match: 'vize_l0::' + MODULES[match[1]] if match[1] in MODULES else match[0], original)
        changed = changed.replace('use vize_davinci::{', 'use vize_l0::{')
        if source.name == 'accept.rs' and crate.name == 'vize_extension_contract':
            changed = changed.replace(' as davinci;', ' as diag;').replace('davinci::', 'diag::')
        if changed != original:
            source.write_text(changed)
    manifest = crate / 'Cargo.toml'
    original = manifest.read_text()
    changed = re.sub(r'^vize_davinci\s*=.*\n', '', original, flags=re.M)
    if changed != original:
        manifest.write_text(changed)
