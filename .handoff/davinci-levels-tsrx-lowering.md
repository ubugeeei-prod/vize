# Handoff: davinci-levels-tsrx-lowering (input-neutral levels record, step 5)

The coordinator closed #13263, because drafts are not allowed. This branch is
stacked on `davinci-levels-tag-facts` (#13250), which is stacked on
`davinci-levels-concepts` (#13232).

## What the branch does

- **Cross-crate moves in the move script.** `level-layout-move.ts` gains
  `crossCrateSteps` and `rewriteCrossCrate`, and its test covers them.
- **`l2-tsrx-lowering` step.** It moves `vize_l2::lang::tsrx` to
  `vize_l1_to_l2::framework::vue::tsrx`.
  - `RegionBuilder::observe` and `RegionBuilder::synthesized_bind` become `pub`.
    This is the only Rust edit made by hand.
  - The storage inventory files the moved files under `l1_to_l2`.
- **`l2-module` step.** It renames `vize_l2::component` to `vize_l2::module`.
- **Decision record.** It records step 5 and how a move proves it is move-only.

Re-running both steps reports `0 changes`. Locally, all of these pass:

- workspace check;
- `vize_l1_to_l2` tsrx tests, 9 of them;
- `vize_l2` lib tests, 303 of them;
- `vize_l4 --features typescript` (`lang::tsrx`, `tsrx_dom`,
  `jsx_module_projection`);
- `vize --test integration tsrx`, 10 tests;
- `vize_patina` tsrx tests;
- clippy;
- the tooling tests.

`tsrx_dom` fails without `--features typescript`. That failure already exists
and is not caused by this branch.

## Blocker: not move-only

I measured with `level-instruction-counts.yml` on CI, rustc 1.100.

| Build                     | Rows that changed | Worst rows                                                                                                          |
| ------------------------- | ----------------- | ------------------------------------------------------------------------------------------------------------------- |
| ci-opt, 16 codegen units  | 28                | all within ±0.16%                                                                                                   |
| one codegen unit          | 20                | `atelier_{dom,ssr,vapor}_compile_stress-interp` +2.1–2.8%; `*_compile_small` +0.4–0.6%; the rest within ±0.005%      |

The one-codegen-unit runs were 44067457219 (base) and 44070585405 (head), both
on temp branches that were deleted afterwards.

TSRX compile itself shows no regression: across all 52 `.tsrx` fixtures × 300,
the output digest is identical and the instruction medians are 2.887e9 → 2.868e9.

The likely cause is that `vize_atelier_dom` (and SSR/Vapor) depend on
`vize_l1_to_l2`. Adding the TSRX lowering to that crate changes how LLVM
partitions and inlines the template-lowering code.

## Next steps (coordinator decision)

1. **Decide on the release profile** (`lto = "fat"`, `codegen-units = 1`).
   Build both base (`davinci-levels-tag-facts`) and head with
   `CARGO_PROFILE_CI_OPT_LTO=fat CARGO_PROFILE_CI_OPT_CODEGEN_UNITS=1`, using a
   separate `CARGO_TARGET_DIR`. Bench `vize_atelier_dom::compile_template_with_options`
   on `tools/benchmarks/crates/davinci_harness/fixtures/stress-interp.vue`
   (its `<template>` body), compiled N times. Take instructions retired from
   `/usr/bin/time -l`.
   - `vize_atelier_dom` has `autotests = false`. Put the temporary test in
     `crates/vize_atelier_sfc/tests/` instead, which depends on
     `vize_atelier_dom`.
   - On CI, an equivalent is a temporary branch whose `[profile.ci-opt]` uses
     fat LTO and one codegen unit, plus one `level-instruction-counts`
     dispatch per side. Do this only when CI is not paused.
2. If the release profile also regresses, restructure so that the Vue-only
   `atelier_*` crates neither link nor inline the TSRX lowering. Either put the
   TSRX lowering in its own crate (for example `vize_l1_to_l2_tsrx`, or under
   `vize_l1_to_l2` behind a feature that atelier does not enable), then
   re-measure.
3. Open a fresh, ready PR only once the counts are proven unchanged, or every
   difference is shown to be noise-free and non-regressing. The PR body draft
   is below.
4. After step 5: step 6, `ModuleDef` / `Edit` / extended `ComponentDef`, with
   the TSRX lowering filling component facts.
   - Coordinate with davinci-inputs (agent `a346c35a56abefb7d`): no new code
     under `vize_l4::lang::tsrx`, and the TSRX lowering now lives at
     `vize_l1_to_l2::framework::vue::tsrx`.

## PR body draft (#13263, closed)

**Title:** refactor(l1_to_l2): move the TSRX lowering beside the JSX lowering,
and name vize_l2::module

Step 5 of `docs/davinci/decisions/2026-11-17-input-neutral-levels.md`. It is a
move with no behavior change:

- The TSRX lowering moves from `vize_l2::lang::tsrx` to
  `vize_l1_to_l2::framework::vue::tsrx`.
- `vize_l2::component` becomes `vize_l2::module`.

The commits are the script changes, the two pub methods, the scripted moves
followed by `cargo fmt`, the storage inventory scope, the docs and the test.
The checks are the ones listed above. Add the instruction-count table from the
measurement once it is resolved.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
