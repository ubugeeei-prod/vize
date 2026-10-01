#!/usr/bin/env python3
"""Replay #6833's final consumer migration and reviewed facade retirement."""
import argparse
import hashlib
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
OLD = Path('davinci/vize_davinci')
NEW = Path('davinci/vize_l0')
MOVES = {
    OLD / 'benches/davinci.rs': NEW / 'benches/pass_runtime.rs',
    OLD / 'benches/davinci_fact.rs': NEW / 'benches/fact_runtime.rs',
}
RETIRED = {
    'Cargo.toml': 'b906d77f5ce09864413d80626f9c6249c45bdf7702d3c5cb403155d34244aa9f',
    'README.md': '41c2091ced415b30fe34260a99297ab367fcfe55905e462adaba6fdfd02dccef',
    'src/lib.rs': 'bbd50ec4193284f1d619be09600a1cd180cfb6170baaf43fd2c8984560b01c2b',
    'src/dump.rs': 'f198a1eac283bbfd14030ffa219961e960a17b219023ef6d05ae62a401508bfe',
    'src/stage.rs': '0a901c9f7bbd3c2d59ab7115f7603293e08c67cf941f92dd71aac17bc6ae23df',
}


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
    present = set()
    for p in (ROOT / OLD).rglob('*'):
        if not p.is_file():
            continue
        rel = p.relative_to(ROOT / OLD).as_posix()
        if OLD / rel in MOVES:
            continue
        if rel not in RETIRED or hashlib.sha256(p.read_bytes()).hexdigest() != RETIRED[rel]:
            raise SystemExit('Refusing changed or unexpected facade file: ' + str(p))
        present.add(rel)
    if present and present != set(RETIRED):
        raise SystemExit('Refusing partially retired facade')
    if phase == 'moves':
        for old, new in MOVES.items():
            if (ROOT / old).exists():
                (ROOT / new).parent.mkdir(parents=True, exist_ok=True)
                subprocess.run(['git', 'mv', str(old), str(new)], cwd=ROOT, check=True)
        return
    if phase == 'check':
        if present:
            raise SystemExit('Facade files remain')
        return
    updates = {}
    def update(path, transform):
        p = ROOT / path
        before = updates.get(p, p.read_text(encoding="utf-8"))
        updates[p] = transform(before)
    def imports(s):
        s = re.sub(r'\bvize_davinci::', 'vize_l0::', s)
        return s.replace('vize_l0::diagnostic', 'vize_l0::diag').replace(
            'vize_l0::witness', 'vize_l0::diag::verify')
    for root in ('crates', 'davinci', 'tools', 'tests/fuzz/fuzz_targets'):
        for p in (ROOT / root).rglob('*.rs'):
            if 'fixtures' in p.parts or 'target' in p.parts or ROOT / OLD in p.parents:
                continue
            update(p.relative_to(ROOT), imports)
    for p in (ROOT / 'crates').glob('*/Cargo.toml'):
        def manifest(s):
            has_l0 = re.search(r'^vize_l0(?:\s*=|\.workspace\s*=)', s, flags=re.M)
            return re.sub(r'^vize_davinci(?:\s*=|\.workspace\s*=)[^\n]*\n',
                          '' if has_l0 else lambda m: m[0].replace('vize_davinci', 'vize_l0'), s, flags=re.M)
        update(p.relative_to(ROOT), manifest)
    update('crates/vize_vitrine/build.rs', lambda s: s.replace(
        '        "davinci/vize_davinci/src",\n',
        '        "davinci/vize_l0/Cargo.toml",\n'
        '        "davinci/vize_l0/src",\n'
        '        "davinci/vize_l0_derive/Cargo.toml",\n'
        '        "davinci/vize_l0_derive/src",\n'))
    update('crates/vize_croquis/src/croquis/vir.rs', lambda s: s.replace(
        'davinci/vize_davinci/tests/croquis_dump.rs', 'crates/vize_croquis/tests/croquis_dump.rs'))
    for backend in ('dom', 'ssr', 'vapor'):
        update(f'crates/vize_atelier_{backend}/tests/davinci_walk_baseline.rs',
               lambda s: s.replace('The plans live in `vize_davinci`',
                                   'The plans live in `vize_curator::legacy_plan`'))
    update('tests/tooling/davinci-storage-policy.test.ts', lambda s: s.replace(
        '  if (file.startsWith("davinci/vize_davinci/")) return "infra";\n', ''))
    update('Cargo.toml', lambda s: re.sub(r'^vize_davinci = [^\n]*\n', '',
           s.replace('  "davinci/vize_davinci",\n', ''), flags=re.M))
    def foundation_manifest(s):
        if 'criterion.workspace' not in s:
            s = s.replace('[dev-dependencies]\n', '[dev-dependencies]\ncriterion.workspace = true\n')
        if 'name = "pass_runtime"' not in s:
            s = s.replace('[[test]]\n', '''[[bench]]
name = "pass_runtime"
harness = false

[[bench]]
name = "fact_runtime"
harness = false

[[test]]
''', 1)
        return s
    update(NEW / 'Cargo.toml', foundation_manifest)
    for old, new in MOVES.items():
        def bench(s):
            s = s.replace(str(old), str(new))
            s = s.replace('-p vize_davinci --bench davinci_fact', '-p vize_l0 --bench fact_runtime')
            s = s.replace('-p vize_davinci --bench davinci', '-p vize_l0 --bench pass_runtime')
            s = re.sub(r'\bdavinci_fact_group\b', 'fact_runtime_group', s)
            s = re.sub(r'\bdavinci_group\b', 'pass_runtime_group', s)
            s = re.sub(r'\bdavinci_fact\b', 'fact_runtime', s)
            s = re.sub(r'\bdavinci\b(?=\s*\()', 'pass_runtime', s)
            return re.sub(r'(criterion_group!\(pass_runtime_group,\s*)davinci\b', r'\1pass_runtime', s)
        update(new, bench)
    update('tools/benchmarks/scripts/instruction-counts.mjs', lambda s: s.replace(
        '["vize_davinci", "davinci"]', '["vize_l0", "pass_runtime"]').replace(
        '["vize_davinci", "davinci_fact"]', '["vize_l0", "fact_runtime"]'))
    def instruction_identities(s):
        for old, new in MOVES.items():
            s = s.replace(str(old), str(new))
        for old, new in (
            ('5119f62005613a194a828943ca2d9a6fb00d2db8cdde3bed2867d504b74a3c42',
             'de92f1a3da8d0ec2e85d94062bddfbcd9cdd1a8775f0b0ed12915ccf692651a1'),
            ('c454e98437ad2107d849c99236e9a7f79712cbc74dbb36b458500a5b3d9a4704',
             'c60dc63e4e974698c11052c32c4e147b58f0695e4a674aec49f47199408fceb4'),
        ):
            s = s.replace(old, new)
        if '# Runtime owner identities:' not in s:
            s = s.replace('davinci_fact_query_observed =',
                '# Runtime owner identities: three identical measurements, numeric ceilings unchanged.\n'
                '# Source: 001f3eee417b1522392233bf79b00f8fe32cf3d7; Actions run 36778058361.\n'
                'davinci_fact_query_observed =', 1)
        return s
    update('docs/davinci/plan/instruction-budgets.toml', instruction_identities)
    update('tools/moon/cmd/publish_crates/main.mbt', lambda s: s.replace('  "vize_davinci",\n', ''))
    update('tools/support/compat/davinci/lib/consumer-migration-scan.mjs', lambda s: s.replace(
        '  stageSurface("davinci", "Davinci", "vize_davinci"),\n', ''))
    for path in ('.github/workflows/davinci-incremental.yml', '.github/workflows/davinci-contracts.yml'):
        update(path, lambda s: s.replace('      - "davinci/vize_davinci/**"\n', ''))
    update('.github/workflows/check.yml', lambda s: s.replace('-p vize_davinci ', '-p vize_l0 ')
           .replace('Davinci no_std portability lanes (TS-24)', 'Level library portability lanes (TS-24)'))
    for path in ('tools/commands/ci/fuzz/seed_corpus.rs', 'tools/support/compat/fuzz/seed_corpus.mjs'):
        update(path, lambda s: s.replace(str(OLD / 'tests/fixtures'), str(NEW / 'tests/fixtures')))
    for path in ('tests/tooling/davinci-storage-policy.test.ts', 'tests/tooling/davinci-module-layout.test.ts'):
        update(path, lambda s: re.sub(r'^  "davinci/vize_davinci(?:/src)?",\n', '', s, flags=re.M))
    update('docs/content/stability.md', lambda s: re.sub(
        r'^\| `vize_davinci`[^\n]*\n', '', s, flags=re.M))
    for p in (ROOT / 'docs').rglob('*.md'):
        def links(s):
            def link(m):
                value = m[0]
                if value.startswith(('http:', 'https:')):
                    return value
                for old, new in MOVES.items():
                    value = value.replace(str(old), str(new))
                for rel in RETIRED:
                    target = NEW / rel
                    value = value.replace(str(OLD / rel), str(target))
                return value
            return re.sub(r'(?<=\]\()[^)]*', link, s)
        update(p.relative_to(ROOT), links)
    for p, changed in updates.items():
        if p.read_text(encoding="utf-8") != changed:
            p.write_text(changed, encoding="utf-8")
    if present:
        subprocess.run(['git', 'rm', *[str(OLD / rel) for rel in RETIRED]], cwd=ROOT, check=True)


if __name__ == '__main__':
    main()
