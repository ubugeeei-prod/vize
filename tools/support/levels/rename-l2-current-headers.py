#!/usr/bin/env python3
"""Replay only the bounded current L2 consumer headers; codecs are authored separately."""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess

BASE = "cc794f7c5"
REPLACEMENTS = (
    (b"[disegno]", b"[l2-dump-v2]"),
    (b"[disegno.ops]", b"[l2-dump-v2.ops]"),
    (b"s2-provenance-folio", b"l2-provenance-dump-v2"),
)
PATHS = (
    'crates/vize_curator/tests/spolvero_ladder.rs',
    'crates/vize_l1_to_l2/tests/css_bind_append.rs',
    'crates/vize_l1_to_l2/tests/css_bind_lowering.rs',
    'crates/vize_l1_to_l2/tests/css_bind_sfc.rs',
    'crates/vize_l1_to_l2/tests/legacy_lowering.rs',
    'crates/vize_l1_to_l2/tests/legacy_pass.rs',
    'crates/vize_l1_to_l2/tests/lowering_battery.rs',
    'crates/vize_l1_to_l2/tests/lowering_elements.rs',
    'crates/vize_l1_to_l2/tests/lowering_html.rs',
    'crates/vize_l1_to_l2/tests/lowering_shapes.rs',
    'crates/vize_l1_to_l2/tests/lowering_vcloak.rs',
    'crates/vize_l1_to_l2/tests/lowering_vtext.rs',
    'crates/vize_l1_to_l2/tests/snapshots/cfg_pass_snapshot__constructs_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/cfg_pass_snapshot__dashboard_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/hoist_pass_snapshot__the_levels_fixture_snapshots_the_post_pass_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/hoist_pass_snapshot__the_positions_fixture_snapshots_the_post_pass_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/text_pass_snapshot__the_condense_fixture_snapshots_the_post_pass_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/text_pass_snapshot__the_merge_fixture_snapshots_the_post_pass_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/vfor_pass_snapshot__the_holes_fixture_snapshots_the_post_pass_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/vfor_pass_snapshot__the_loops_fixture_snapshots_the_post_pass_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/vif_pass_snapshot__the_chain_fixture_snapshots_the_post_pass_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/vif_pass_snapshot__the_collision_fixture_snapshots_the_post_pass_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/vmodel_pass_snapshot__the_bindings_fixture_snapshots_the_post_pass_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/vmodel_pass_snapshot__the_invalid_fixture_snapshots_the_post_pass_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/vslot_pass_snapshot__the_groups_fixture_snapshots_the_post_pass_folio.snap',
    'crates/vize_l1_to_l2/tests/snapshots/vslot_pass_snapshot__the_invalid_fixture_snapshots_the_post_pass_folio.snap',
    'crates/vize_l2/tests/expr_replay.rs',
    'crates/vize_l2/tests/fixtures/invalid/attr-escapes-element.folio',
    'crates/vize_l2/tests/fixtures/invalid/backwards-span.folio',
    'crates/vize_l2/tests/fixtures/invalid/binding-escapes-owner.folio',
    'crates/vize_l2/tests/fixtures/invalid/branch-escapes-if.folio',
    'crates/vize_l2/tests/fixtures/invalid/child-escapes-branch.folio',
    'crates/vize_l2/tests/fixtures/invalid/child-escapes-element.folio',
    'crates/vize_l2/tests/fixtures/invalid/component-child-escapes.folio',
    'crates/vize_l2/tests/fixtures/invalid/compound.folio',
    'crates/vize_l2/tests/fixtures/invalid/else-mid-chain.folio',
    'crates/vize_l2/tests/fixtures/invalid/empty-if.folio',
    'crates/vize_l2/tests/fixtures/invalid/fallback-escapes-slot.folio',
    'crates/vize_l2/tests/fixtures/invalid/floating-else.folio',
    'crates/vize_l2/tests/fixtures/invalid/leading-else.folio',
    'crates/vize_l2/tests/fixtures/invalid/model-attr-escapes.folio',
    'crates/vize_l2/tests/fixtures/invalid/region-escapes-for.folio',
    'crates/vize_l2/tests/fixtures/reference.folio',
    'crates/vize_l2/tests/folio_cloak.rs',
    'crates/vize_l2/tests/folio_css_bind.rs',
    'crates/vize_l2/tests/folio_html.rs',
    'crates/vize_l2/tests/folio_laws.rs',
    'crates/vize_l2/tests/folio_legacy.rs',
    'crates/vize_l2/tests/folio_model_name.rs',
    'crates/vize_l2/tests/folio_once_memo.rs',
    'crates/vize_l2/tests/folio_rejections.rs',
    'crates/vize_l2/tests/folio_show.rs',
    'crates/vize_l2/tests/folio_slot_model.rs',
    'crates/vize_l2/tests/folio_text.rs',
    'crates/vize_l2/tests/provenance_dump.rs',
    'crates/vize_l2/tests/stage_name_alias.rs',
    'crates/vize_l2/tests/verifier_observer.rs',
    'crates/vize_vitrine/src/wasm/tests_spolvero.rs',
    'playground/src/features/stages/components.test.ts',
    'playground/src/features/stages/folioLines.test.ts',
    'playground/src/features/stages/ladder.test.ts',
    'playground/src/features/stages/panels.test.ts',
    'playground/src/features/stages/provenance.test.ts',
)
FORMATTED = {
    'crates/vize_l2/tests/folio_rejections.rs',
    'crates/vize_l2/tests/provenance_dump.rs',
}


def transform(source):
    for old, new in REPLACEMENTS:
        source = source.replace(old, new)
    return source


def inverse(source):
    for old, new in reversed(REPLACEMENTS):
        source = source.replace(new, old)
    return source


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-ref", default=BASE)
    parser.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    spec = importlib.util.spec_from_file_location("dump_names", Path(__file__).with_name("dump-names.py"))
    names = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(names)
    prepared = []
    changed = 0
    for original in PATHS:
        destination = names.destination(original)
        if original.endswith("/folioLines.test.ts"):
            destination = original.replace("/folioLines.test.ts", "/dumpLines.test.ts")
        if not (root / destination).exists():
            destination = original
        before = subprocess.run(["git", "show", args.base_ref + ":" + original],
                                cwd=root, capture_output=True)
        if before.returncode and destination != original:
            before = subprocess.run(["git", "show", args.base_ref + ":" + destination],
                                    cwd=root, capture_output=True)
        if before.returncode:
            raise SystemExit("missing bounded source: " + original)
        expected = transform(before.stdout)
        if expected == before.stdout:
            continue
        if inverse(expected) != before.stdout:
            raise SystemExit("non-injective header mapping: " + original)
        if original in FORMATTED:
            expected = subprocess.check_output(["rustfmt", "--edition", "2024", "--emit", "stdout"],
                                               input=expected, cwd=root)
        path = root / destination
        current = path.read_bytes()
        if args.verify:
            if current != expected:
                raise SystemExit("current source differs from bounded transformation: " + destination)
        elif current not in (before.stdout, expected):
            raise SystemExit("source changed outside bounded transformation: " + destination)
        else:
            prepared.append((path, expected))
        changed += 1
    # Preflight every selected path before the first write.
    for path, expected in prepared:
        path.write_bytes(expected)
    print(json.dumps({"base": args.base_ref, "selectedPaths": len(PATHS), "transformedPaths": changed,
                      "inverseBytesEqual": True, "formattingPaths": sorted(FORMATTED),
                      "writes": 0 if args.verify else len(prepared), "verified": args.verify}))


if __name__ == "__main__":
    main()
