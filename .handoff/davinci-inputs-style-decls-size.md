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
