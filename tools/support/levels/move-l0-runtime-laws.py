#!/usr/bin/env python3
"""Replay #6833 foundation law ownership; preserve captured fixture bytes."""
import argparse
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
OLD = Path('davinci/vize_davinci')
NEW = Path('davinci/vize_l0')
RELATIVE = (
    "artifact_keys.rs",
    "dump_collector.rs",
    "dump_remarks.rs",
    "fact_alpha/groups.rs",
    "fact_alpha/main.rs",
    "fact_demand/main.rs",
    "fact_demand/words.rs",
    "fact_manager/groups.rs",
    "fact_manager/main.rs",
    "fact_preserve/main.rs",
    "fact_preserve/numbers.rs",
    "fixtures/keys/base-l2-recipe3.keys",
    "fixtures/keys/base-v1.keys",
    "fixtures/keys/base.keys",
    "fixtures/keys/base.vue",
    "fixtures/keys/capture-v2/current-build.stderr",
    "fixtures/keys/capture-v2/current-build.stdout",
    "fixtures/keys/capture-v2/current-initial-hash_domains.stderr",
    "fixtures/keys/capture-v2/current-initial-hash_domains.stdout",
    "fixtures/keys/capture-v2/current-key_manifests.stderr",
    "fixtures/keys/capture-v2/current-key_manifests.stdout",
    "fixtures/keys/capture-v2/current-rebuilt-build.stderr",
    "fixtures/keys/capture-v2/current-rebuilt-build.stdout",
    "fixtures/keys/capture-v2/current-rebuilt-global_summary.stdout",
    "fixtures/keys/capture-v2/current-rebuilt-key-tests.stdout",
    "fixtures/keys/capture-v2/current-rebuilt-key_manifests.stdout",
    "fixtures/keys/capture-v2/current-rebuilt-sfc_summary.stdout",
    "fixtures/keys/capture-v2/current-rebuilt-ts43-update.stderr",
    "fixtures/keys/capture-v2/current-rebuilt-ts43-update.stdout",
    "fixtures/keys/capture-v2/current-rebuilt-vector-observer.stderr",
    "fixtures/keys/capture-v2/current-rebuilt-vector-observer.stdout",
    "fixtures/keys/capture-v2/old-build.stderr",
    "fixtures/keys/capture-v2/old-build.stdout",
    "fixtures/keys/capture-v2/old-observer-1.stderr",
    "fixtures/keys/capture-v2/old-observer-1.stdout",
    "fixtures/keys/capture-v2/old-observer-2.stderr",
    "fixtures/keys/capture-v2/old-observer-2.stdout",
    "fixtures/keys/capture-v2/old-observer-3.stderr",
    "fixtures/keys/capture-v2/old-observer-3.stdout",
    "fixtures/keys/capture-v2/old-observer-4.stderr",
    "fixtures/keys/capture-v2/old-observer-4.stdout",
    "fixtures/keys/capture-v2/one-shot-compile.stderr",
    "fixtures/keys/capture-v2/one-shot-input.tsv",
    "fixtures/keys/capture-v2/one-shot-pinned-compile.stderr",
    "fixtures/keys/capture-v2/one-shot-pinned-compile.stdout",
    "fixtures/keys/capture-v2/one-shot-pinned-rustc-version.stdout",
    "fixtures/keys/capture-v2/one-shot-rustc-version.stdout",
    "fixtures/keys/capture-v2/one-shot-verification.stderr",
    "fixtures/keys/capture-v2/one-shot-verification.stdout",
    "fixtures/keys/capture-v2/one_shot.rs",
    "fixtures/keys/capture-v2/receipt.json",
    "fixtures/keys/hash-domains-v2.json",
    "folio_derive_laws.rs",
    "folio_dump.rs",
    "fusion_plan_dump.rs",
    "hash_domains.rs",
    "key_manifests.rs",
    "pass_observer_law.rs",
    "pass_observer_timing.rs",
    "pass_pipeline_syntax.rs",
    "remark_channel.rs",
    "remark_corpus.rs",
    "remark_dump.rs",
    "remark_zero_cost.rs",
    "witness_law.rs",
    "witness_tiers.rs",
    "witness_verify/facts.rs",
    "witness_verify/main.rs",
)
MOVES = {OLD / 'tests' / name: NEW / 'tests' / name for name in RELATIVE}


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
    updates = {}
    def update(path, transform):
        p = ROOT / path
        before = updates.get(p, p.read_text())
        updates[p] = transform(before)
    for name in RELATIVE:
        if name.endswith('.rs') and not name.startswith('fixtures/'):
            update(NEW / 'tests' / name, lambda s: re.sub(r'\bvize_davinci::', 'vize_l0::', s)
                   .replace('vize_l0::diagnostic', 'vize_l0::diag')
                   .replace('vize_l0::witness', 'vize_l0::diag::verify'))
    def manifest(s):
        if 'vize_atelier_sfc = { path' not in s:
            addition = '''# Foundation laws use higher-level and legacy producers only as dev oracles.
# Version-less path declarations are stripped when this foundation is packaged.
davinci_harness.workspace = true
vize_atelier_sfc = { path = "../../crates/vize_atelier_sfc" }
vize_l1 = { path = "../vize_l1" }
vize_l1_to_l2 = { path = "../vize_l1_to_l2" }
vize_l2 = { path = "../vize_l2" }
'''
            assert s.count('[dev-dependencies]\n') == 1
            s = s.replace('[dev-dependencies]\n', '[dev-dependencies]\n' + addition)
        if 'name = "remark_zero_cost"' not in s:
            s = s.replace('[lints]\n', '''[[test]]
name = "remark_zero_cost"
# Exclude libtest's reporting thread from process-wide allocation counters.
harness = false

[lints]
''')
        return s
    update(NEW / 'Cargo.toml', manifest)
    def old_manifest(s):
        start, end = s.index('[dev-dependencies]'), s.index('[[bench]]')
        s = s[:start] + '[dev-dependencies]\ncriterion = { workspace = true }\ndavinci_harness = { workspace = true }\n\n' + s[end:]
        return re.sub(r'\[\[test\]\]\nname = "remark_zero_cost"\n[^[]*', '', s)
    update(OLD / 'Cargo.toml', old_manifest)
    for path in (
        '.github/workflows/davinci-incremental.yml',
        '.github/workflows/davinci-unused-bindings.yml',
        'tests/tooling/davinci-incremental-workflow.test.ts',
        'docs/davinci/plan/test-suites.md',
    ):
        update(path, lambda s: s.replace('-p vize_davinci --test', '-p vize_l0 --test'))
    update('.github/workflows/davinci-unused-bindings.yml', lambda s: s if '"davinci/vize_l0/**"' in s else
           s.replace('    paths:\n', '    paths:\n      - "davinci/vize_l0/**"\n', 1))
    for path in ('tests/tooling/compiled-docs-inputs.test.mjs', 'tests/tooling/support/davinci-phase2-ledger.ts',
                 'docs/davinci/plan/key-manifests.md'):
        update(path, lambda s: s.replace(str(OLD / 'tests'), str(NEW / 'tests')))
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
    # All path and manifest rewrites are buffered before writing any integration.
    for p, changed in updates.items():
        if p.read_text() != changed:
            p.write_text(changed)


if __name__ == '__main__':
    main()
