#!/usr/bin/env python3
"""Replay #6833 crash-report ownership in the inspection library."""
import argparse
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
OLD = Path('davinci/vize_davinci')
NEW = Path('crates/vize_curator')
MOVES = {
    OLD / 'src/dump/repro.rs': NEW / 'src/repro.rs',
    OLD / 'tests/repro_dump.rs': NEW / 'tests/repro_dump.rs',
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
    update(NEW / 'src/repro.rs', lambda s: s.replace('crate::dump', 'vize_l0::dump').replace('crate::pass', 'vize_l0::pass'))
    update(NEW / 'src/lib.rs', lambda s: s if 'pub mod repro;' in s else s + '\npub mod repro;\n')
    for base in (ROOT / 'crates', ROOT / 'tests/fuzz/fuzz_targets'):
        for p in base.rglob('*.rs'):
            if 'fixtures' in p.parts or 'snapshots' in p.parts:
                continue
            update(p.relative_to(ROOT), lambda s: s.replace('vize_davinci::dump::repro', 'vize_curator::repro'))
    update(NEW / 'tests/repro_dump.rs', lambda s: s.replace('vize_davinci::dump', 'vize_l0::dump'))
    update(OLD / 'src/dump.rs', lambda s: s.replace('pub mod repro;\n', ''))
    update('tests/fuzz/Cargo.toml', lambda s: s if 'vize_curator = ' in s else
           s.replace('vize_davinci = ', 'vize_curator = { path = "../../crates/vize_curator" }\nvize_davinci = '))
    for p in (ROOT / 'tests/fuzz/fuzz_targets').glob('*.rs'):
        update(p.relative_to(ROOT), lambda s: re.sub(r'\bvize_davinci::(dump|pass|side_table)\b', r'vize_l0::\1', s)
               .replace('and the repro page (`vize_davinci`).', 'and the repro page (`vize_curator`).'))
    update('tests/fuzz/Cargo.toml', lambda s: re.sub(r'^vize_davinci\s*=.*\n', '', s, flags=re.M))
    update('docs/davinci/plan/storage-inventory.tsv', lambda s: '\n'.join(
        row for row in s.splitlines() if str(OLD / 'src/dump/repro.rs') not in row) + '\n')
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
    update('docs/davinci/plan/folio-format.md', lambda s: s.replace('davinci/vize_davinci/tests/repro_dump.rs', 'crates/vize_curator/tests/repro_dump.rs'))


if __name__ == '__main__':
    main()
