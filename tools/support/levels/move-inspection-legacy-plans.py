#!/usr/bin/env python3
"""Replay #6833 historical backend-plan ownership in the inspection library."""
import argparse
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
OLD = Path('davinci/vize_davinci/src/legacy_plan.rs')
NEW = Path('crates/vize_curator/src/legacy_plan.rs')
HEADER = '''//! Historical template-traversal plans for inspected legacy compiles.
//!
//! The DOM, SSR and Vapor plans record the template walks pinned by the
//! backends' `davinci_walk_baseline` laws. Each backend reads this inspection
//! metadata through a path-only dev dependency. The CLI uses the same plans
//! to describe its selected legacy compile when writing a crash report.
//!
//! The declarations describe traversals; the real pass bodies run in their
//! backend owners. These plans retain the historical two-barrier baseline and
//! do not attribute native per-pass execution. Vapor generation walks Vapor
//! IR and Croquis walks the script AST, so both remain outside this template
//! traversal baseline. See `docs/davinci/plan/walk-baseline.md`.

'''


def update(path, transform):
    p = ROOT / path
    original = p.read_text()
    changed = transform(original)
    if changed != original:
        p.write_text(changed)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('phase', choices=('moves', 'integrate', 'check'))
    phase = parser.parse_args().phase
    source, target = ROOT / OLD, ROOT / NEW
    if source.exists() and target.exists():
        raise SystemExit('Refusing source/target collision: ' + str(OLD))
    if not source.exists() and not target.exists():
        raise SystemExit('Missing source and target: ' + str(OLD))
    if phase != 'moves' and source.exists():
        raise SystemExit('Unmoved source: ' + str(OLD))
    if phase == 'moves':
        if source.exists():
            subprocess.run(['git', 'mv', str(OLD), str(NEW)], cwd=ROOT, check=True)
        return
    if phase == 'check':
        return
    update(NEW, lambda s: HEADER + s[re.search(r'^use (?:crate|vize_l0)::pass::', s, flags=re.M).start():].replace('crate::pass', 'vize_l0::pass'))
    update('crates/vize_curator/src/lib.rs', lambda s: s if 'pub mod legacy_plan;' in s else s + '\npub mod legacy_plan;\n')
    update('davinci/vize_davinci/src/lib.rs', lambda s: re.sub(r'//! - \[`legacy_plan`\].*\n//!   plans.*\n', '', s).replace('pub mod legacy_plan;\n', ''))
    for crate in ('vize', 'vize_atelier_dom', 'vize_atelier_ssr', 'vize_atelier_vapor'):
        for p in (ROOT / 'crates' / crate).rglob('*.rs'):
            if 'fixtures' not in p.parts:
                update(p.relative_to(ROOT), lambda s: s.replace('vize_davinci::legacy_plan', 'vize_curator::legacy_plan')
                       .replace('vize_atelier_core::legacy_plan', 'vize_curator::legacy_plan'))
        if crate != 'vize':
            update(Path('crates') / crate / 'Cargo.toml', lambda s: s if 'vize_curator =' in s else
                   s.replace('[dev-dependencies]\n', '[dev-dependencies]\n# Inspection metadata is a dev-only, unpublished oracle; strip it on packaging.\nvize_curator = { path = "../vize_curator" }\n'))
    for p in (ROOT / 'docs').rglob('*.md'):
        def links(s):
            def link(m):
                value = m[0]
                return value if value.startswith(('http:', 'https:')) else value.replace(str(OLD), str(NEW))
            return re.sub(r'(?<=\]\()[^)]*', link, s)
        update(p.relative_to(ROOT), links)


if __name__ == '__main__':
    main()
