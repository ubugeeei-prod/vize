#!/usr/bin/env python3
"""Replay #6833 stage-feed ownership without changing schema or JSON bytes."""
import argparse
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
OLD = Path('davinci/vize_davinci')
NEW = Path('crates/vize_curator')
MOVES = {
    OLD / 'src/dump/feed.rs': NEW / 'src/inspector/feed.rs',
    OLD / 'tests/stage_feed.rs': NEW / 'tests/stage_feed.rs',
}


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
    for old, new in MOVES.items():
        source, target = ROOT / old, ROOT / new
        if source.exists() and target.exists():
            raise SystemExit('Refusing source/target collision: ' + str(old))
        if not source.exists() and not target.exists():
            raise SystemExit('Missing source and target: ' + str(old))
        if phase != 'moves' and source.exists():
            raise SystemExit('Unmoved source: ' + str(old))
    if phase == 'moves':
        for old, new in MOVES.items():
            if (ROOT / old).exists():
                (ROOT / new).parent.mkdir(parents=True, exist_ok=True)
                subprocess.run(['git', 'mv', str(old), str(new)], cwd=ROOT, check=True)
        return
    if phase == 'check':
        return
    update(NEW / 'src/inspector/feed.rs', lambda s: s.replace('crate::dump', 'vize_l0::dump').replace('crate::pass', 'vize_l0::pass')
           .replace('IO-free (`no_std + alloc`) and', 'IO-free and'))
    update(NEW / 'src/lib.rs', lambda s: s if 'extern crate alloc;' in s else s.replace('pub mod complexity;', 'extern crate alloc;\n\npub mod complexity;'))
    update(NEW / 'src/inspector.rs', lambda s: s if 'pub mod feed;' in s else s.replace('mod diff;', 'mod diff;\npub mod feed;'))
    for p in (ROOT / NEW / 'src').rglob('*.rs'):
        update(p.relative_to(ROOT), lambda s: s.replace('pub use vize_davinci::dump::feed', 'pub use super::feed')
               .replace('feed shape `vize_davinci` owns', 'feed shape this inspector owns')
               .replace('vize_davinci::dump', 'vize_l0::dump').replace('vize_davinci::pass', 'vize_l0::pass'))
    update(NEW / 'tests/stage_feed.rs', lambda s: s.replace('vize_davinci::dump::feed', 'vize_curator::inspector::feed')
           .replace('vize_davinci::dump', 'vize_l0::dump').replace('vize_davinci::pass', 'vize_l0::pass'))
    update(OLD / 'src/dump.rs', lambda s: s.replace('pub mod feed;\n', ''))
    update(NEW / 'Cargo.toml', lambda s: s.replace('# The Spolvero feed shape (P2-18) and the L1 surface tree its inspector\n# pages are proven through; both `no_std + alloc`, so the wasm consumer\n# (`vize_vitrine --features wasm`) is untouched.\nvize_davinci = { workspace = true }\n', '# The L1 surface tree observed by the inspector.\n'))
    update('docs/davinci/plan/storage-inventory.tsv', lambda s: '\n'.join(
        row for row in s.splitlines() if str(OLD / 'src/dump/feed.rs') not in row) + '\n')
    for p in (ROOT / 'docs').rglob('*.md'):
        def links(s):
            def link(m):
                value = m[0]
                if value.startswith(('http:', 'https:')):
                    return value
                for old, new in MOVES.items():
                    value = value.replace(str(old), str(new))
                return value
            return re.sub(r'(?<=\]\()[^)]*', link, s)
        update(p.relative_to(ROOT), links)
    update('docs/davinci/plan/test-suites.md', lambda s: s.replace('-p vize_davinci --test stage_feed', '-p vize_curator --test stage_feed'))
    update('tests/tooling/support/davinci-phase2-ledger.ts', lambda s: s.replace(
        str(OLD / 'tests/stage_feed.rs'), str(NEW / 'tests/stage_feed.rs')))


if __name__ == '__main__':
    main()
