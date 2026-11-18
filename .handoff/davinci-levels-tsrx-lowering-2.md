# Handoff: davinci-levels-tsrx-lowering-2 (input-neutral levels, step 5)

## State

The branch is based on main 8d4b97a5aa and has no PR. It holds:
- the move-script cross-crate support and its test (cherry-picked, with the
  test conflict resolved);
- `RegionBuilder::observe` and `synthesized_bind` made `pub`;
- the scripted `l2-tsrx-lowering` move (`vize_l2::lang::tsrx` →
  `vize_l1_to_l2::framework::vue::tsrx`), which on current main also rewrites
  the import in `crates/vize_patina/src/linter/engine/tsrx.rs`;
- the storage inventory scope `l1_to_l2`;
- the scripted `l2-module` rename;
- the decision record docs.

Re-running both steps reports 0 changes.

## Move-only proof (done)

- All 2,078 `.vue` files compile byte-identically base vs head in DOM, SSR and
  Vapor, with the same 60 failures on both sides.
- The 54 `.tsrx` files give identical build output (DOM/SSR/Vapor) and
  identical lint JSON.
- The atelier, l1_to_l2, l2, l4 (typescript), `vize` tsrx integration and
  patina tsrx suites pass. `component_slot_text` and `array_program_projection`
  fail on base too.

## Release-profile measurements

These use `level-instruction-counts.yml` with ci-opt set to fat LTO and one
codegen unit, base and head each run twice, interleaved. A/A is 0.

1. **Move alone vs main** (runs 44279931693/44279957408 base,
   44279943716/44279969909 head): `atelier_{dom,ssr,vapor}_compile_stress-interp`
   +2.87/+2.42/+2.18%, and `*_compile_small` +0.4–0.6%.
   - Cause: `compact_str` `Repr::push_str` stops being inlined into
     `vize_l2::analysis::text_facts::rebuild_source` once the TSRX code leaves
     `vize_l2`.
   - So neither option (a) (own crate) nor option (b) (cargo feature) can fix
     it: `vize_l2` would be identical to head either way.
   - PR #13340 (`perf/l2-rebuild-source`) fixes it.
2. **#13340 + move vs #13340** (runs 44283817356/44283858071 vs
   44283830068/44283869365), deterministic, A/A 0: there are still 5 tiny
   regressions.
   - `atelier_dom_compile_large` +0.0126%, `atelier_dom_compile_medium`
     +0.0114%, `s1_to_s2_lower_vfor_three_aliases` +0.0098%,
     `atelier_ssr_compile_stress-interp` +0.0034%,
     `atelier_ssr_compile_small` +0.0015%.
   - Earlier callgrind diffs attribute them to code-layout changes in
     `vize_l1_to_l2`: `pass::vslot::spell::fold` is outlined, and
     `vslot::consume::bindings` and `lower_children_guarded` shift.
   - The `sfc-instructions` rows differ between A/A twins, so they are noise
     there.

## Next steps

1. Once #13340 is decided and merged, rebase this branch on main. The move
   commits do not touch `text_facts.rs`. Re-run both scripted steps; they must
   report 0 changes.
2. Re-measure under the release codegen with A/A. Per the coordinator, open
   the `refactor(l2)` PR only if every row is within A/A noise. Expected
   outcome: the 5 rows above still regress by about 0.01%, so stop and report
   unless the coordinator accepts them.
3. Gates still owed: CI clippy, the assertion lint, the source-length check
   against the merge base, the dependency allowlists, `completion-gates.ts`, and
   the `levels.inputs` owed count.

## Open questions

- Is a deterministic +0.01% on 5 rows acceptable for a move-only refactor?
  If not, the vslot/lower layout sensitivity in `vize_l1_to_l2` needs its
  own perf fix first, the same way #13340 handled `rebuild_source`.
