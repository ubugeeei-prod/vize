# Handoff: #13338 size / Ir follow-up (davinci-inputs-style-decls), 2026-11-19

Remove this `.handoff/` directory before anything from this branch merges.
PR #13338 head is unchanged (old, not rebased); auto-merge is OFF. This WIP branch holds the
rebased PR (onto main 9c7d4cba55) plus two perf commits:

- `460e1911c6` = old 592cdb20 (`CssLintResult::warn` used by every css/* rule).
- `05631b398b` keeps `registered_sfc::{has_active, run}` `#[inline(never)]` and puts `#[inline]` on
  the markup visitors `walk_children` / `walk_attributes` / `walk_modifier_ranges`.
- Rebase conflicts were only the three explain-oracle sha256 rows in
  `tests/_fixtures/differential/linter/manifest.json`; they are set to the merged fixture hashes
  (CI's differential check has not run on this head yet).

## Bars, base = main 9c7d4cba55 (Linux x86, my driver `perf/pb-driver` + scratch screen)

| candidate | release bytes | ci-opt bytes | level rows | lint Ir lanes (6) |
| --- | ---: | ---: | --- | --- |
| 05631b398b (tree = bench/sd2-cB2) | -2,432 | -104,512 | 2 up: `patina_jsx_markup_one_root` +293 (+0.707%), `patina_s2_markup_one_root` +430 (+0.094%); 99 identical; A/A 0 | all down 0.13-0.25% (bench default -9.2M, reka default -6.2M) |
| 460e19 + pin only (sd2-cB1) | -2,432 | -68,736 | 2 down (-188, -206) | all UP ~0.04-0.12% |
| 71f936 (no warn, sd2-cA) | +1,792 | -32,512 | 2 down | mixed, opinionated/nuxt up |
| warn in structure lane only (sd2-cC/C1) | +512 | -10,240 | s2_markup +0.76% | mixed |

- The visitor `#[inline]` hints are what flip the lint lanes, and that is deterministic: every
  hinted variant passes all 6 lanes (b1h, b4h, t3, t5, t6, t7, cB2, cC2). No subset of the three
  hints passes (h1-h4 all fail).
- The same hints cost the two markup bench rows the same +293/+430 in every variant.
- Main + hints alone (sd2-base2) is better on 4 lanes and +0.8-1.0M on bench opinionated/nuxt.
- Findings parity, ci-profile CLI, misskey/n8n × happy-path/opinionated/nuxt:
  - head is byte-identical to 71f936 (= old 8d72ebe008 rebased).
  - Against main the only change is `css/prefer-logical-properties` +670 (misskey) and +940
    (n8n), 0 removed, and happy-path is identical. That is the same set as the PR body.
- Gates on the head: fmt, CI clippy (vize_patina), assertion lint, source lengths, storage policy,
  storage summary and docs-all-lint-rules (16/16) all pass.
  - The vize_patina suite passes except
    `lint_history::full_current_results_and_independent_fix_applications`, which fails identically
    on main locally (pre-existing).

## Next steps
1. Find why the hints add about 300 instructions on the tiny markup benches. A callgrind diff of
   the `davinci_markup` bench is wanted: the scratch workflow `bench/sd2-cgb` ran the bench without
   the harness env. Run it with instruction-counts-lib.mjs `GUEST_ENVIRONMENT`
   (`VIZE_INSTRUCTION_COUNTS=1`, the mimalloc vars and `GLIBC_TUNABLES`). Then either move the hint
   to the specific rule call sites (NoConsecutiveBr, NoDeprecatedSlotAttribute,
   NoDeprecatedVOnNativeModifier), or offset the markup-pass cost.
2. Once a variant has every level row ≤ main, re-run the release size, lanes, the quiet-runner A/B
   + A/A (`pr-performance-observation.yml --ref bench/davinci-inputs-style-lane`) and the PR
   observation. Then push to the PR branch with the tables and arm auto-merge.
3. Delete the temp branches `bench/sd2-*`.

## Update (per-site inlining, maintainer decision: no metric trade-offs)
- Candidates pushed, all on top of `sd2-cB1` (warn + registered_sfc pin, no global hints), and
  compile-checked. Each splits `walk_children` / `walk_attributes` / `walk_modifier_ranges` into
  the plain public fn and an `#[inline(always)] pub(crate) fn *_inlined`. The public fn calls
  `*_inlined`, so the walk body is unchanged. Only the three rule sites call `*_inlined`:
  `html/no_consecutive_br`, `vue/no_deprecated_slot_attribute` and
  `vue/no_deprecated_v_on_native_modifier`.
  - `bench/sd2-p1` f5110f3827: all three walks.
  - `bench/sd2-p5` 96148d57bc: p1 without the `walk_children` site.
  - `bench/sd2-p6` 7d25172d15: p1 without the registered_sfc pin.
- Not measured yet: the CI queue stayed at 100-150 (> 40) from 09:30 to 10:10 UTC. Dispatch when it
  drops:
  - `gh workflow run pr-performance-observation.yml --ref bench/davinci-style-decls-size -f shas="9c7d4cba558cf7f7dbab6d92b47f939ce1b664a3 f5110f38274f418cb9df4a2b9094cf5d561ddd48 96148d57bcb300b474319124e71d05e6f4ca0897 7d25172d15f226e36f67634ec1180e75512ffb1c"` (lint Ir lanes)
  - `gh workflow run level-instruction-counts.yml --ref perf/pb-driver -f release=true -f roles='["base=9c7d4cba558cf7f7dbab6d92b47f939ce1b664a3","p1=f5110f38274f418cb9df4a2b9094cf5d561ddd48","p5=96148d57bcb300b474319124e71d05e6f4ca0897","p6=7d25172d15f226e36f67634ec1180e75512ffb1c"]'` (rows + release size)
  - Markup-bench callgrind diff, now with the harness guest env (driver fixed at 71d2801bdd):
    `gh workflow run level-instruction-counts.yml --ref bench/sd2-cgb -f shas="base=9c7d4cba558cf7f7dbab6d92b47f939ce1b664a3 cB1=4061e9b0676d350efb8e82835eb29b2c703c1e60 cB2=f8d83cb58c8cb3c7dc9dc76444bc1c8a21ffd18e"`.
    Diff its `*.cg*.txt` with
    `/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/pb/style/cgdiff.py`
    (it expects `<role>-<lane>.txt` names; adapt for `<role>.cg.<n>.txt`).
- Remaining temp branches: `bench/sd2-{cB1,cB2,cg,cgb,p1,p5,p6}`.

## Per-site results (screen 44510858938, driver 44510865349): all fail
| cand | release | lint lanes vs main (bd/bo/bn/rd/ro/rn, M Ir) | markup rows |
| --- | ---: | --- | --- |
| p1 | -2,176 | +2.80/+4.10/+4.22/-1.60/-4.38/-4.21 | jsx +130 (+0.31%), s2 -1,403 |
| p5 | -2,176 | +3.02/-0.03/+0.52/+0.72/-2.43/-1.82 | jsx -59, s2 +502 (+0.11%) |
| p6 | -2,688 | ≈ p1 | ≈ p1 |
Also `sfc_parse_template_only` +5 in all three (it reads 3320/3325 across rebuilds of identical
source, so it is probably noise, but it counts). Per-site inlining does not reproduce the
global-hint lane win on the bench-1000 lanes. Untried: the markup-bench callgrind diff (bench/sd2-cgb, guest env fixed).
