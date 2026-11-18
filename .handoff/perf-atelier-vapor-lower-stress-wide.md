# perf/atelier-vapor-lower-stress-wide handoff (2026-11-18)

PR #13344 (ready, auto-merge NOT armed). Branch now holds variant v5 (sources) on top of the
earlier v1 commit; the PR body still describes v1 and needs updating.

## Root causes
- `atelier_vapor_lower_stress-wide` +1.38% (#13307): no table work runs on this fixture. #13307
  (l3.rs now instantiates `lower_with_caps_and_comment_policy::<Vec<_>>` in the vapor crate)
  re-partitioned the ci-opt build (thin LTO, 16 CGUs). `HashMap<&str,()>::contains_key` stopped
  being inlined into `write_attributes`, so each of the 100 static-attr override probes rose to
  ~64 instr (+1405). Fix: skip the probe when no `v-bind` argument has the attr's length
  (u32 mask, bit len%32). v5 builds the mask by scanning `el.props` in `write_attributes` only
  (no struct/type change).
- `atelier_ssr_codegen_large` +0.10% (#13328): `static_slot_name` built an owned String for every
  static slot, and `slot_entry_name` quoted that copy. v5: `SlotName::{Plain(&str), Dotted(String)}`
  borrowed in `ComponentTemplateSlot.name`; `slot_entry_name` match arm calls
  `#[inline(always)] quoted_static_slot_name` that quotes the argument directly; dotted paths
  are `#[cold] #[inline(never)]`.

## Measurements (level-instruction-counts.yml; Callgrind is deterministic)
- Main 403207f2d8: run 44289548336. A/A on 8d4b97a5: 44276729662 (merge queue) vs 44286294812
  = 0 diffs over 107 stages.
- Pre-culprit references: lower_stress-wide 102055 (fbe3f910); ssr_codegen_large 801474
  (ef44203d, merge-queue run 44254143442). Main: 103460 / 802276.
- v1 (PR head ecfeb925, run 44292501940): wide 97129, ssr 801722 (> pre); up: lower_stress-interp
  +776, compile_small +52, compile_medium +73.
- v2 (44294305604): 11 vapor stages up (generate_imports outlined). Rejected.
- v3 (44296595392): wide 97172, ssr 801481; up: generate/compile_stress-wide +300
  (generate_operation +3/call, same callees = partition), lower_stress-interp +1.
- v4a/v4b (44299016749/44299215348): SSR quoted helper returning Option<String> is worse (801530).
- v5 (44300504570, bench branch perf-bench/atelier-vapor-lower-v5 = this branch's sources):
  ssr 801411 (<= pre-culprit OK), wide 98529 (OK); still up: generate_stress-wide +300,
  compile_stress-wide +300, lower_large +56, generate_large +44, compile_large +100.

## Status of gates
- Output byte-identity was verified for v1 only (DOM/SSR/Vapor of 6,222 inputs: 0 diffs). Redo for v5.
- v5: cargo fmt + CI clippy (--profile ci -D warnings -D clippy::wildcard_imports) clean; tests
  for ssr/vapor/sfc NOT yet completed on v5 (v1 passed). Assertion lint, source-length vs merge
  base, storage tests passed on v1; component.rs stays at 659 lines in v5.

## Next steps
1. The SSR half (v5) meets both bars. Keep it.
2. Vapor: every variant moves unrelated vapor-crate codegen (CGU re-partition under ci-opt), e.g.
   `generate::operations::generate_operation` +3 instr/call with identical callees. Decide with
   the user whether such partition-only flips can be accepted (the vapor change is a real
   -4.8..-6.1% on the target stage), or split SSR into its own PR and look into vapor separately
   (e.g. check generate_operation asm on x86_64 to see what changed).
3. Before pushing again: run tests, the byte-identity dump (scripts were in the session
   scratchpad; vize build --format json --continue-on-error over all .vue + .vue.txt,
   modes dom/--ssr/--vapor), update the PR body, then `gh pr merge 13344 --auto`.
4. Delete bench branches perf-bench/atelier-vapor-lower-{v2,v4b,v5} after.
