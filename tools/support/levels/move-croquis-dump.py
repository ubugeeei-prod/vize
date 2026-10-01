#!/usr/bin/env python3
"""Replay #6833 Croquis page ownership and its byte-exact fixture laws."""
import argparse
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
OLD = Path('davinci/vize_davinci')
NEW = Path('crates/vize_croquis')
MOVES = {
    OLD / 'src/dump/croquis.rs': NEW / 'src/dump.rs',
    OLD / 'src/dump/croquis': NEW / 'src/dump',
    OLD / 'tests/fixtures/croquis': NEW / 'tests/fixtures/croquis',
    **{OLD / ('tests/' + name + '.rs'): NEW / ('tests/' + name + '.rs')
       for name in ('croquis_dump', 'dump_croquis', 'folio_laws', 'pipeline_snapshot')},
    **{OLD / ('tests/snapshots/' + name): NEW / ('tests/snapshots/' + name)
       for name in ('folio_laws__folio_snapshot_macro_uses_the_normalized_full_printer.snap',
                    'pipeline_snapshot__a_pipeline_run_snapshots_the_full_normalized_folio.snap')},
}


def update(path, transform):
    path = ROOT / path
    original = path.read_text()
    changed = transform(original)
    if changed != original:
        path.write_text(changed)


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
    sources = [NEW / 'src/dump.rs'] + [p.relative_to(ROOT) for p in (ROOT / NEW / 'src/dump').rglob('*.rs')]
    for p in sources:
        update(p, lambda s: s.replace('crate::dump::croquis', 'crate::dump')
               .replace('crate::dump::{Dump,', 'vize_l0::dump::{Dump,')
               .replace('crate::dump::Mode', 'vize_l0::dump::Mode')
               .replace('crate::dump::Error', 'vize_l0::dump::Error'))
    update(NEW / 'src/lib.rs', lambda s: s if 'pub mod dump;' in s else
           s.replace('// Core modules', 'extern crate alloc;\n\n// Core modules')
           .replace('pub mod drawer;', 'pub mod drawer;\npub mod dump;'))
    for base in (ROOT / NEW / 'src', ROOT / NEW / 'tests'):
        for p in base.rglob('*.rs'):
            def migrate(s):
                s = s.replace('vize_davinci::dump::croquis', 'vize_croquis::dump')
                s = re.sub(r'\bvize_davinci::(fact|pass|dump|assert_dump_snapshot)\b', r'vize_l0::\1', s)
                s = s.replace('vize_atelier_core::Allocator', 'vize_l0::Allocator')
                s = s.replace('vize_atelier_core::parser', 'vize_armature::parser')
                s = s.replace('-p vize_davinci --test croquis_folio', '-p vize_croquis --test croquis_dump')
                return s.replace('davinci/vize_davinci/src/folio.rs', 'davinci/vize_l0/src/dump.rs')
            update(p.relative_to(ROOT), migrate)
    update(OLD / 'src/dump.rs', lambda s: s.replace('pub mod croquis;\n', ''))
    def manifest(s):
        s = s.replace('vize_davinci.workspace = true', 'vize_l0.workspace = true')
        dev = re.search(r'^\[dev-dependencies\]\n(.*?)(?=^\[|\Z)', s, flags=re.M | re.S)
        if dev is not None and 'vize_atelier_sfc = ' in dev[1]:
            return s
        start = s.index('# Dev-only cycle')
        end = s.index('\n[[bench]]', start)
        dependency = s[start:end].replace('the davinci bench needs', 'the fixture harness and host bench need')
        s = s[:start] + s[end:]
        return s.replace('[dev-dependencies]\n', '[dev-dependencies]\n' + dependency + '\n')
    update(NEW / 'Cargo.toml', manifest)
    update('tests/fuzz/fuzz_targets/folio_parse.rs', lambda s: s
           .replace('vize_davinci::dump::croquis', 'vize_croquis::dump')
           .replace('vize_davinci::dump::{Dump,', 'vize_l0::dump::{Dump,'))
    update('tests/fuzz/Cargo.toml', lambda s: s if 'vize_croquis = ' in s else
           s.replace('vize_davinci = ', 'vize_croquis = { path = "../../crates/vize_croquis" }\nvize_davinci = '))
    update('tools/commands/ci/fuzz/seed_corpus.rs', lambda s: s if '"crates/vize_croquis/tests/fixtures/**/*.folio"' in s else
           s.replace('const FOLIO_GLOBS: &[&str] = &[', 'const FOLIO_GLOBS: &[&str] = &[\n    "crates/vize_croquis/tests/fixtures/**/*.folio",'))
    update('docs/davinci/plan/storage-inventory.tsv', lambda s: '\n'.join(
        row for row in s.splitlines() if str(OLD / 'src/dump/croquis') not in row) + '\n')
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
    for p in ('docs/davinci/plan/test-suites.md', 'docs/davinci/plan/folio-format.md'):
        update(p, lambda s: s.replace('-p vize_davinci --test croquis_folio', '-p vize_croquis --test croquis_dump')
               .replace('-p vize_davinci --test dump_croquis', '-p vize_croquis --test dump_croquis'))


if __name__ == '__main__':
    main()
