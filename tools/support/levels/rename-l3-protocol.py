#!/usr/bin/env python3
"""Apply the bounded current L3 graph protocol rename from its frozen parent."""
import argparse
import json
from pathlib import Path
import subprocess
import re

BASE = '851a77d26a3478c32939ca206ab09a4fc217e5ca'
REPLACEMENTS = (
    (r'impeto\.', r'l3\.'),
    ('s3-folio', 'l3-dump-v2'),
    ('unknown Impeto op kind', 'unknown L3 op kind'),
    ('unknown Impeto edge kind', 'unknown L3 edge kind'),
)
FORMATTED = {
    'docs/davinci/plan/folio-format-impeto.md',
    'docs/davinci/plan/impeto-ops.md',
    'playground/src/features/stages/folioLines.test.ts',
    'tests/tooling/davinci-impeto-ops-doc.test.ts',
}
PATHS = (
    'crates/vize_curator/tests/spolvero_ladder.rs',
    'crates/vize_davinci/tests/folio_derive_laws.rs',
    'crates/vize_davinci_derive/src/model.rs',
    'crates/vize_l2_to_l3/tests/snapshots/l3_snapshots__control_and_slots_template_snapshots_l3_folio.snap',
    'crates/vize_l2_to_l3/tests/snapshots/l3_snapshots__static_and_dynamic_template_snapshots_l3_folio.snap',
    'crates/vize_l2_to_l3/tests/snapshots/placement_snapshots__different_reads_and_root_statics_offer_nothing.snap',
    'crates/vize_l2_to_l3/tests/snapshots/placement_snapshots__identical_direct_reads_offer_a_group.snap',
    'crates/vize_l2_to_l3/tests/snapshots/placement_snapshots__loop_items_group_but_never_cache.snap',
    'crates/vize_l2_to_l3/tests/snapshots/placement_snapshots__static_branch_content_offers_a_hoist_and_handlers_a_cache.snap',
    'crates/vize_l3/src/dump.rs',
    'crates/vize_l3/src/op/kind.rs',
    'crates/vize_l3/tests/folio_laws.rs',
    'crates/vize_l3/tests/op_family.rs',
    'crates/vize_vitrine/src/wasm/tests_spolvero.rs',
    'docs/davinci/plan/folio-format-impeto.md',
    'docs/davinci/plan/impeto-ops.md',
    'playground/e2e/davinci-ladder.test.ts',
    'playground/src/features/stages/folioLines.test.ts',
    'playground/src/features/stages/ladder.test.ts',
    'playground/src/features/stages/ladder.ts',
    'playground/src/features/stages/partition.test.ts',
    'playground/src/features/stages/partition.ts',
    'tests/formal/impeto/Impeto/Folio.lean',
    'tests/formal/impeto/Impeto/Syntax.lean',
    'tests/formal/impeto/fixtures/control-flow.s3.folio',
    'tests/formal/impeto/fixtures/dynamic-button.s3.folio',
    'tests/formal/impeto/fixtures/ivm-matrix.lowered.jsonl',
    'tests/formal/impeto/fixtures/model-loop-reference.lowered.jsonl',
    'tests/formal/impeto/fixtures/model-reference.lowered.jsonl',
    'tests/formal/impeto/fixtures/rust-lowered-control-slots.s3.folio',
    'tests/formal/impeto/fixtures/rust-lowered-loop-keyed.s3.folio',
    'tests/formal/impeto/fixtures/rust-lowered-loop-nested.s3.folio',
    'tests/formal/impeto/fixtures/rust-lowered-loop-unkeyed.s3.folio',
    'tests/formal/impeto/fixtures/rust-lowered-static-dynamic.s3.folio',
    'tests/formal/impeto/fixtures/slot-reference.lowered.jsonl',
    'tests/formal/impeto/fixtures/static-text.s3.folio',
    'tests/tooling/davinci-impeto-ops-doc.test.ts',
    'tests/tooling/davinci-ivm-matrix.test.ts',
)

OPCODES = ('set-prop', 'set-dynamic-props', 'set-text', 'set-event', 'set-html',
           'set-template-ref', 'insert-node', 'prepend-node', 'directive', 'if',
           'for', 'create-component', 'slot-outlet', 'get-text-child', 'child-ref',
           'next-ref', 'missing')


def transform(source):
    source = re.sub(r'\bimpeto\.(' + '|'.join(sorted(OPCODES, key=len, reverse=True)) + r')\b',
                    lambda match: 'l3.' + match[1], source)
    for old, new in REPLACEMENTS:
        source = source.replace(old, new)
    return source

def inverse(source):
    source = re.sub(r'\bl3\.(' + '|'.join(sorted(OPCODES, key=len, reverse=True)) + r')\b',
                    lambda match: 'impeto.' + match[1], source)
    for old, new in reversed(REPLACEMENTS):
        source = source.replace(new, old)
    return source

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--node", default="node")
    parser.add_argument("--base-ref", default=BASE)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[3]
    changed = []
    prepared = []
    for name in PATHS:
        before = subprocess.check_output(["git", "-C", str(repo), "show", args.base_ref + ":" + name], text=True)
        expected = transform(before)
        if expected == before:
            continue
        if inverse(expected) != before:
            raise SystemExit("non-injective mapping: " + name)
        if name in FORMATTED:
            formatted = subprocess.check_output(
                [args.node, str(repo / 'node_modules/oxfmt/bin/oxfmt'), '--stdin-filepath=' + name],
                input=expected, text=True, cwd=repo, stderr=subprocess.PIPE)
            normalize = lambda text: re.sub(r',(?=[}\])])', '', re.sub(r'\s+', '', re.sub(r'(?m)^\s*\|[ |:-]+\|\s*$', '', text)))
            if normalize(formatted) != normalize(expected):
                raise SystemExit('format changed semantic tokens: ' + name)
            expected = formatted
        path = repo / name
        if args.verify:
            if path.read_text() != expected:
                raise SystemExit("current source differs from frozen transformation: " + name)
        else:
            if path.read_text() not in {before, expected}:
                raise SystemExit("source changed outside bounded transformation: " + name)
            prepared.append((path, expected))
        changed.append(name)
    for path, expected in prepared:
        path.write_text(expected)
    print(json.dumps({"base": args.base_ref, "files": len(changed), "rawTransformInverseByteEqual": True, "currentEqualsFormattedTransform": True, "layoutOnlyFormattingFiles": sorted(FORMATTED), "verified": args.verify, "paths": changed}))

if __name__ == "__main__":
    main()
