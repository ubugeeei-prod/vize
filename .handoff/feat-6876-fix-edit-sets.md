# Lane E (linter and fixes) handoff, 2026-11-19

## Open PRs

- #13397 `feat/6848-lint-substrate-ledger` (LN1): generated per-rule substrate ledger
  `docs/davinci/plan/lint-substrate-ledger.tsv`, `--check` lane in `source-inventories.ts`,
  `lint-substrate.md` in the run bundle, record `2026-11-19-6848-lint-substrate-ledger.md`.
  Tooling only; auto-merge on.
- #13399 `docs/6876-one-fix-type` (FX1): record `2026-11-19-6876-one-fix-type.md`. Docs only; auto-merge on.
- #13406 `feat/6876-fix-edit-sets` (FX2, step 1): all lint fix appliers go through
  `vize_l1::edit::{select_first, EditSet}`. Parity probe 0/87,003 result diffs, 0/22,605 fix diffs.
  **Closed as not ready: it grows the binary.**
  - Release profile (fat LTO, 1 CGU), stripped `vize`, local probe: base `f16ac2dff9`
    56,753,616 B (sha256 75f2e8db...) → head `f9b8548d95` 57,199,504 B (753e2930...), **+445,888 B**.
  - PR performance observation run 44496791416, ci-opt: 79,786,840 → 80,818,072 B (+1,031,232).
    Wall times were stable. RSS moved ±1 MB on every lane, including Compile and Format, which this
    change does not touch.
  - The code change is about 100 lines, so this is a codegen or inlining cascade, not new code.
    First rule out build noise with an A/A release build of the base. Main's ci-opt size moved
    about 1 MB between `dc783944a3` (80,784,152) and `f16ac2dff9` (79,786,840), so the base
    number itself may be the outlier. Use the release-measure workflow for both builds.
    The next step is to build both with `--config 'profile.release.package.vize.strip="none"'` and
    diff `nm --size-sort` to find the function that grew.
  - Suspects: the `sort_by_key` monomorphization of `SpanEdit<'_, '_, ()>` (48 bytes) in
    `select_first`, the new `EditSet<()>` instantiations, and the `#[inline] Fix::apply` body change.
    One restructuring to try: select over `&TextEdit` (the old CLI sorted pointer-sized items) and
    bind `SpanEdit`s only after selection.
  - No instruction-count dispatch was made, because the CI queue was over 100.

## Maintainer decision (2026-11-19, later): pure L1 for syntax rules

Syntax rules run on the pure L1 `native/` engine with no L2 lowering. The L2 markup facade serves
semantic rules only. #13397's ledger already targets `L1 syntax` at `native-l1` and adds an
`l1-callback` column (12 of 202 today). Instruction counts are judged on the RELEASE profile
(fat LTO, 1 CGU), every row ≤ base, with ci-opt informational. Release size must not grow. Use the
perf-backlog agent's dispatchable release-measure workflow, not local release builds. As of
2026-11-19 it had not landed on main.

LN2 replaces item 2 below:
1. Complete `native/` until it admits every input the facade admits, with no whole-input
   refusals. The refusal kinds to retire are in `native.rs` (`NativeLintRefusal`) and
   `native/template.rs` (`NativeTemplateLintRefusal`, the attribute and text profiles).
2. Make `native/` the default engine for syntax rules. A run with both syntax and semantic rules
   builds L1 once and lowers L2 once from it (`L2Template::lower` already starts from the one L1
   parse), with no second parse. A syntax-only run never lowers L2.
3. Migrate the 202 `L1 syntax` ledger rows onto it in waves of about 20, by directory. Start with
   the 12 `l1-callback = yes` rows, then the 82 `markup-l2` syntax rows, then the 111
   `relief-visitor` rows. Each wave deletes the rule's Relief/facade body in the same PR, so no
   wave runs two implementations.

## Next steps, in order (pre-decision; item 2 is superseded by the section above)

1. **FX2 step 2:** move `Fix`/`TextEdit` from `vize_patina::diagnostic::types` to `vize_l0::diag`
   (serde is already a vize_l0 dependency) and re-export them from Patina under the current paths.
   Keep the field names and serde derive identical. `Fix::apply` cannot stay an inherent method
   once the type is in L0, because L0 cannot reach `vize_l1::edit`. Either keep `apply` in Patina as
   a free function that the tests and observers call, or add an L1 extension trait. Prove the JSON
   is byte-identical with the CLI `--format json` on the lint differential fixtures.
2. **LN2 wave 1:** standalone HTML (`linter/engine/standalone.rs`) is the only production path that
   still runs the Relief `Rule` bodies of the 86 `markup-l2` rules (`TemplateRuleEnv::relief`, with
   `facade_rules: &[]`). The steps:
   - Switch standalone to `sfc::facade::RULES`, with parity over every `.html` in the corpora.
     The L2 template is already lowered there, so this adds no parse.
   - Then delete the Relief `enter_*` bodies of the facade rules, by directory (html/, ecosystem/,
     vapor/ first, then a11y/, then vue/).
   - The SFC battery (`linter/engine/sfc/facade/tests.rs::compare_template`) compares the facade
     against `lint_template_relief`. Pin its current outputs as snapshots before the bodies go,
     or the oracle disappears with them.
   - Each wave regenerates the ledger, so the rows move from `legacy-rule` to `native`.
3. **LN2 waves 2+:** port the 111 `relief-visitor` rules (ledger rows with engine `relief-visitor`)
   to `MarkupRule` and add them to `sfc::facade::RULES`, about 20 per PR by directory. Follow
   #13315's pattern and evidence: real-project corpus plus fixtures, with default, `with_all` and
   all-fallback configs, byte-identical. Once no `relief-visitor` rule remains, `with_compatibility_root`
   goes from Patina, and the Relief allowlist edge goes with it.
4. **LN5:** the 8 `css-lightningcss` rules move to `StyleRule`. #13338 (another agent) already moves
   prefer-logical-properties, require-font-display and prefer-slotted.
5. LN3 waits for lane A's `vize_l2::query` Q3/Q4, LN4 for the L1 program profile for scripts, and
   LN7 for JSX J9.

## Notes

- Patina's `native/` lane is opt-in, and only tests and observer examples call it. The ledger does not
  count it (see the LN1 record). Fold it into the facade or delete it as part of LN2.
- `lint_history::full_current_results_and_independent_fix_applications` fails on macOS at
  `multiline-v-for-source-9880` (diagnostic count, type-aware environment) on origin/main too.
  To check the remaining cases locally, filter that id out of the loop temporarily.
- Worktrees do not carry hydrated `tests/_fixtures/_git` checkouts. Read the main checkout's copies
  (about 2,100 files).
