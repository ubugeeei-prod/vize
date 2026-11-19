# Handoff: refactor/sfc-descriptor-l1 (#6851 products.dependencies)

Paused 2026-11-19 (fleet stop). This file must be deleted before merge.

## State

The branch is based on main `650fea85fd` and has no PR yet. Commits:

1. `602181af29`: drop the unused Relief `SfcScriptBlock::bindings`. It is always `None`.
2. `0243308195`: update the test expectation for the descriptor's Rust serde packet
   (`crates/vize_atelier_sfc/tests/unicode_script_line_comments.rs` loses `"bindings": null`).
   No shipped output serializes `SfcDescriptor`: napi and wasm build their own JSON.
3. `bb8fd3289b`: drop the unused Relief `SfcParseOptions::template_parse_options`. The fixture
   observers keep emitting `"template_parse_options": null`.
4. `77d55a8dba`: move `SfcDescriptor`, its blocks, `SfcParseOptions`/`SfcError`/`parse_sfc*` and the
   `v-bind()` scanner to `vize_l1::container::vue::compat`. Croquis re-exports them under the old
   paths. `SfcError`'s `From<CompilerError>` becomes `vize_atelier_sfc::types::sfc_error_from_compiler`.
   `vize_incremental` reads the descriptor from L1, so its normal `vize_croquis` edge goes and
   Maestro's allowlist drops to 18 (gate: 35 edges). Musea's block split also reads L1.
5. `b054b076c2`: the descriptor tests live as L1 unit modules (`compat/tests/`). The assertion
   ratchet rows were renamed with the same counts.
6. Canon's `legacy` re-export now points at `vize_l1::dialect` (part of commit 1). The version model
   already lived in L1; Armature only re-exported it.

Local checks, all green: workspace `cargo check --all-targets`; tests for vize_l1 lib (compat 136),
croquis lib, atelier_sfc lib, incremental and musea; CI clippy and the clippy test-target lint; fmt;
assertion lint; source lengths; the dependency checker (`unlisted: []`, `stale: []`); dependency
tooling tests 48/48.

Of the `vize_atelier_sfc` integration tests, 68 fail locally only because node_modules are missing
(happy-dom, @babel/core, vue-vapor-runtime). CI must confirm them.

## In-flight CI

- Probe **44492961984** on `tmp/pd3-probe` (it replaces `level-instruction-counts.yml` and must not
  be merged). It builds base `650fea85fd` and head `b054b076c2` in thin and fat (release profile).
  Read its `parity-probe` artifact:
  - `size.txt`: the fat stripped bytes must be equal (release-size rule).
  - `parity.txt`: `vize build` DOM/SSR/Vapor JSON over all 2,079 tracked .vue files must be
    byte-identical, and `vize lint -f json` must be identical.
- Not yet dispatched (the queue was above 40): `level-instruction-counts.yml` on `tmp/pd3-ic-base`,
  `tmp/pd3-ic-head`, then `tmp/pd3-ic-base-twin` and `tmp/pd3-ic-head-twin`.
  Compare with `python3 iccmp.py base head base-twin head-twin`; the script is described in the
  product-deps handoff.

## Known risk: ci-opt instruction shifts

On commits 1–3 alone (runs 44476255413 / 44476259867), 20 rows moved:
- `croquis_analyze_*` up to +0.10%;
- `patina_jsx_markup_one_root` +0.71% and `patina_s2_markup_one_root` +0.13%;
- `musea_parse_art_*` −0.01 to −0.04%;
- `sfc_parse_template_only*` −0.3 to −0.8%.

The callgrind diffs show inlining and register shifts in unrelated functions, caused by the
thin-LTO, 16-CGU partition (for example `MarkupElement::walk_attributes` closures, `Drawer::*`),
not added work. The full instruction set must be measured on the final head. If rows regress:
- Either remove the inlining sensitivity (for example `#[inline]` on the small generic patina
  `walk_*` helpers),
- or ask whether instruction rows are also judged on the release profile, as size is.

## Next steps (in order)

1. Read probe 44492961984. Run the four instruction runs. Open the PR with tables in the format of
   #13375 and #13340, and enable auto-merge only with release-size evidence and no instruction
   regression.
2. Musea `defineArt` fact: add a new file in `davinci/vize_l2/src/framework/vue/` (lane A owns the
   directory; keep the change in a new file and rebase onto their PRs). It must equal Croquis
   `parse_define_art`/`parse_define_art_meta` (`crates/vize_croquis/src/script_parser/define_art.rs`
   has the battery) and expose the `ArtDefinition` fields. Musea also uses `parse_script_setup` for
   import ranges, the `defineArt` span and binding names (`transform/to_csf_script.rs`, `parse.rs:113`).
   Switch those to the L2 `SetupFacts` bindings and macro calls and the L2 Import rows, then drop
   `vize_musea -> vize_croquis` (−1 Maestro row).
3. Then follow the inventory order on `handoff/product-deps`: #6885 step 9 (JSX), #6844 S8
   (Patina), #6849 (Canon), #6850 (Maestro).
