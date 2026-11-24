## What

This is the readability pilot (lanes L4a, plus one L3 rename) on six quiet crates: `vize_fresco`, `vize_musea`, `vize_doctor`, `vize_marquette`, `vize_curator` and `vize_armature`. When it was opened, no open PR touched any of them. Every edit is mechanical or a reviewed deletion, and the house-style rules it applies are in `docs/project/source-style.md` (#<PR2>).

| Rule | Change | Count |
| --- | --- | ---: |
| No issue numbers in comments (§1.3) | removed from 20 comment lines in `vize_armature`; the sentences keep their reason | 20 → 0 |
| No issue numbers in names (§1.2) | `crates/vize_armature/tests/sfc_attribute_boundary_11731.rs` → `sfc_attribute_boundary.rs` (nothing referenced the name) | 1 → 0 |
| Delete comments that restate the code (§1.3) | 99 reviewed deletions (see below) | 141 → 42 flagged |
| Import groups std / external / local (§1.5 rule 5) | 69 leading `use` blocks regrouped by script, then `cargo fmt` sorted within groups | 69 → 0 files |
| One blank line between items (§1.5 rule 1) | 29 blank lines after a closing `}` | 29 sites |

Some restating comments were **kept** on purpose:
- every doc comment in `vize_doctor` (it denies `missing_docs`);
- `vize_marquette` docs, which add a word of meaning;
- `vize_fresco/src/napi/**`, where docs are shipped `index.d.ts` text;
- label series in long builders and enums (`vrt/preset.rs` "Desktop/Tablet/Mobile", `palette/codegen.rs`, `docs/{catalog,markdown}.rs`);
- docs that carry information beyond the name ("arena-allocated", "as JSON", "in Markdown").

`crates/vize_armature/src/parser/tests.rs` (1,966 lines) gained a blank import-group line, so one restating comment in it was deleted to keep it from growing.

Not in this PR: moving the 30 sibling `*_tests.rs` / `tests_*.rs` files and the 2 `mod.rs` files (class T/N1). Their parents' `super::` paths change, and `parser/tests.rs` is over the limit, so they get their own PR.

## Evidence (class N0, plus one class T rename)

| Check | Result |
| --- | --- |
| Rust token identity, 105 changed `.rs` files (`rust-token-identity.ts --base origin/main`) | 10 identical, 56 `use-reordered`, 38 `doc-only` (none on clap/napi/wasm-bindgen items), 1 moved (the test rename), 0 different; 100 comments removed |
| IR identity with panic locations normalized (`ir-identity-locations.ts`, release profile, 6 crates) | <IR> |
| Release size, `shipped-profile-measure.yml` base/head (`vize` CLI and `vize_vitrine` N-API) | <SIZE> |
| Tests `cargo test -p <6 crates>` / CI clippy `--profile ci -D warnings -D clippy::wildcard_imports` | <TESTS> |
| Test-list parity | the rename keeps every `#[test]` name; only the integration-test binary name changes |
| `cargo fmt --check` | clean |
| `source-file-lengths.ts --check --base-ref origin/main` | no new or grown file over 500 |
| Byte custody | no registered source touched (the crates' only rows are 4 record-only `Cargo.toml` digests) |
| Readability ratchet | baseline lowered with `--write` for the six areas |

## How to regenerate

The scripts live outside the repository; a later lane folds them into `tools/commands/readability/`:
- `regroup_imports.py <files>` regroups the leading `use` block by kind and keeps each group's order, then `cargo fmt -p …` runs;
- `item_spacing.py <files>` adds a blank line between `}` and a following item at the same indentation;
- the issue-number edits and the restating deletions were reviewed by hand from the ratchet's candidate list.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
