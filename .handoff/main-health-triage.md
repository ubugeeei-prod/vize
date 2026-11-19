# main-health triage: daily full Check on main (paused 2026-11-19)

Delete this file before merging. This branch (`ci/full-check-alert`) also carries
the alert guard commit (see "Fix in progress").

## Scope of the red

- The scheduled Check has been red **every day since 2026-10-18**. The last green
  schedule run was 39014647393 (2026-10-17, 9ca34e084a). It is not only since 11-15.
- Runs read: 44465592435 (11-19 dispatch, e0e4d5a656, latest complete), 44295421831
  (11-18), 43705442769 (11-15), 39109598041 (10-18, first red: nix-flake + vue-parity),
  plus job lists of every schedule run from 10-17 to 11-18 (first/last red per job below).
- 44515288203 (11-19 schedule, 8b09dea8b1) was still `queued` at pause time. Jobs already
  red: full-js-original-project, vue-parity, coverage, v1-supported-matrix, source-coverage,
  Full Rust workspace and doctests, **editor-host-smoke (new)**, rust-davinci-allocations,
  **Tooling (differential-lsp-history + 69 files) (new shard)**.
- Aggregators (`clippy-and-test`, `full-rust-report`, `test-scripts`, `test-js-packages`,
  `full-js-report`) only restate producer failures. Ignore them.

## Triage table (latest run 44465592435)

| Job | Red since | Cause | Suspected PR | Class | Fix |
| --- | --- | --- | --- | --- | --- |
| rust-davinci-allocations | 11-19 (green 11-18 c1c890d4cc) | `patina_jsx_markup_one_root` allocs 0->1, `patina_s2_markup_one_root` 7->8 (exact budgets.toml gate) | #13351 `perf(patina): preplan markup rule callbacks` (27c5bb3eb8). `MarkupRules::new` now builds 6 SmallVec plans (inline 64/8/8/8/8/8); one spills past 8 = 1 heap alloc per lint call. The bench measures `MarkupRules::new` inside the window. Also check #13348/#13356/#13358/#13359 in c1c890d4cc..e0e4d5a656 | **real perf regression** (one alloc per linted file in production: engine.rs:381, engine/markup.rs:25, sfc/facade.rs:176, tsrx.rs:46) | Confirm locally: `cargo bench -p vize_patina --bench davinci_markup -- --quick` at 27c5bb3eb8^ and 27c5bb3eb8. Then size the spilling SmallVec or build the plan without heap. Never raise the budget. `perf(patina)` PR with level-instruction-counts + release size |
| Full Rust workspace and doctests | current cause since 11-16 (11-15 had a different, now-fixed TS40 name mismatch) | `tests/differential/typechecker-workspace.ts:157` `requiredCases`: registry expects `library_generic_props_fallback::old_library_generic_props_follow_the_unknown_props_policy` in the `vize_canon::fix_history_diagnostics` binary, but no such test exists there (only a lib unit test in `src/batch/type_checker/tests/recent_issues/library_generic_props_fallback.rs` with a different name) | #13028 (a83ca382b0) registered it; #13106 (6723581166) added the pack `tests/_fixtures/differential/typechecker/library-generic-props-fallback-13021` (4 cases) without a runner test | **test bug** (registry vs. test binary) | Add `crates/vize_canon/tests/library_generic_props_fallback/mod.rs` with `fn old_library_generic_props_follow_the_unknown_props_policy` calling `check_pack("library-generic-props-fallback-13021", "<name above>", "f06d427619f943bf1192cfafbae65de537e464d0", Some(13021), &["loose-fallthrough-valid","declared-generic-prop-stays-inferred","reported-default-loose","reported-strict"])` (pattern: `tests/aria_kebab_prop_11048/mod.rs`) + `mod library_generic_props_fallback;` in `tests/fix_history_diagnostics.rs`. Verify with `cargo nextest list` that the full expected list matches (the log diff hides lines). Test-only, product binary unchanged |
| vue-parity | 10-18 (x32) | Snapshot mismatches `tests/snapshots/check/__snapshots__/{compiler-macros,style-preprocessors,ecosystem-products}-check.snap` + typecheck-errors digest after #10950/#11064/#11049. Actual virtual TS: setup body emitted verbatim (no 2-space re-indent, `@vize-map` range shrinks), `$ComponentConstructor` wrapped in `$IsAny/$IsUnknown ... NoInfer<...>`, new `readonly __vizeFallthroughSource?:` member | Likely #12572 (setup-verbatim + constructor NoInfer, 11-10), #13119 (d554a022a4, `__vizeFallthroughSource`, 11-16). Fork A did not report back | most likely **stale snapshots** (deliberate ledgered changes; vue-parity runs only on schedule/dispatch). Digest change still needs a per-diagnostic check | Verify each change is ledgered in its PR, then regenerate with the snapshot tool's official update mode after a `--profile ci` build. Never re-pin blind |
| coverage | 11-11 (x8) | `target/ci/coverage`: `sfc/script-setup: array with as const` fails, SFC 166/167 < budget 167 | not bisected (window 11-10..11-11) | **probable real compiler regression**, needs a check | Bisect 11-10..11-11 compiler commits; `fix(atelier)` with a pinned compiler fixture |
| v1-supported-matrix / Generated fail-closed inventory contract | 11-16 (x3) | "v1 qualification is incomplete: 0/420 required rows; blocking gaps: content-mapper-v1-qualification, case-packet-join, host-runtime-join, linter-rule-partition, vue-baseline-and-maturity, editor-depth" | the PR that added it to the full run (~11-16) | **by-design fail-closed gate**, not a regression | Decide (maintainer): keep it out of the daily green signal or give it an explicit "expected incomplete" mode |
| full-js-original-project | 10-21 (x12) | n8n reference prep: `Package not found in workspace: AbsolutePathBuf("/tmp/vize-n8n-reference-package-…")`, Oxlint qualification 1.78.0/1.86.0 exit 1 | not bisected | probably **env/test harness** (temp package outside the pnpm workspace) | Fork C was on it. Re-triage from the job log |
| source-coverage | 10-30 (x16) | not read (26k-line log) | — | ? | Fork C was on it |
| Tooling shards (8) | various | failing tests: completion via real server, `vize lsp surfaces template completion contexts`, `component prop completion tracks unsaved child edits and repair`, `source-built CSS consumers survive exact originals…`, `shared compiler dom corpus observes 10 complete outputs…`, `named Vue functions retain call hierarchy through (un)opened SFC dependencies`, `source CLI and complete Nuxt/editor publications report original setup reads…`, `shared formatter API history observes complete source-built output…`, docs-render-concurrency-browser (cause not found yet) | not bisected | ? (mix of LSP/compiler/formatter output drift and env) | Fork B was on it. Shard names are bucket heads, not the failing files |
| coverage / sticky disk | — | `Sticky disk limit exceeded (500)` 429 warnings in several jobs | — | **env** (Blacksmith installation quota) | Prune unused sticky disks |
| Nuxt jobs (cancelled) | — | cancelled, reason not checked | — | ? | — |

## Why nobody was alerted

The full suite runs only on `schedule`/`workflow_dispatch`, no PR or merge group waits
on it, and `test-report` runs only for pull_request/merge_group. GitHub mails a
scheduled-workflow failure only to the last editor of the cron line, and no workflow
opened an issue.

## Fix in progress (this branch)

Commit 0652aae9d0 `ci: open a tracking issue while the full Check on main is red`:
- `.github/workflows/check-full-alert.yml`: `workflow_run` on `Check` completed. For
  schedule/dispatch on the default branch it keeps one marked issue open, comments on
  every further red run (failing producer jobs only), and closes the issue on green.
  Runs on hosted ubuntu-24.04 (added to `hostedZeroComputeJobs`) with issues: write only.
- `tools/support/compat/github/full-check-alert.ts` + `tests/tooling/full-check-alert.test.ts`
  (7 tests pass). The workflow catalog was regenerated with
  `node tools/commands/ci/workflow-catalog.ts`. `vp fmt --check` and `vp lint` are clean.
- Branch pushed. **PR not opened yet** (the queue was at 40). Next step: open the PR (the body
  states it is tooling-only and no product binary changes; mention it to the ci-tiers agent
  because it adds a workflow_run listener on `Check`), then enable auto-merge.

## Next steps (in order)

1. Allocation regression (real): bisect, fix in patina, prove it with the bench + instruction rows.
2. Coverage `array with as const`: bisect 11-10..11-11 and fix if it is a regression.
3. Full Rust workspace: add the missing fix-history runner test (above).
4. vue-parity: confirm each change was deliberate, then regenerate the snapshots officially.
5. Tooling shards, full-js-original-project, source-coverage: finish the triage (forks A-C
   were stopped before they reported) and add the two new reds from 44515288203
   (editor-host-smoke, differential-lsp-history shard).
6. v1 matrix: maintainer decision.
Scratch logs (machine-local, may be gone): the session scratchpad `mh/` directory.

## Fork findings that arrived at the pause (partial, nothing bisected to the end)

**vue-parity**
- typecheck-errors digest: red since 11-13 (43412308841). Culprit #12935 (8b2095e36d), which
  deliberately renamed `__DefineProps<__LooseRequired<…>>` to `DefineProps<LooseRequired<…>>`.
  The committed `typecheck-errors-check.snap:59` still has the old text, and the file and
  error counts are unchanged. **Stale pin.** Regenerate it, then update `diagnosticSha256` and
  `snapshotSha256` in `currentReceipt`.
- The 3 check snapshots: setup verbatim, constructor NoInfer and `__vizeFallthroughSource`
  are missing from all committed snaps. **Likely stale.** Regenerate (the update env var is in
  `tests/_helpers/snapshot.ts`) and confirm that only `virtualTs` changes, never diagnostics.

**coverage `array with as const`**
- Red since 11-11 (43006471918; 11-10 42804054033 was green). Likely culprit #12639
  (0f9027744e), "align setup binding kinds with Vue"; other candidates #12699, #12847, #12839.
- Expected output: `tests/expected/sfc/script-setup.snap:1947`. Fixture:
  `tests/fixtures/sfc/script-setup.pkl:879`.
- Still open whether this is a regression or intentional. Diff the actual output first.

**full-js-original-project**
- Red since 11-09 (42608482666). Culprit #12082 (73e378751d), which moved `npm/oxlint/` to
  `npm/linter/oxlint/`. The historical commit e742ca8ee7 only has `npm/oxlint/`.
- **Test bug.** Fix: set `prefix` back to `"npm/oxlint/"` in
  `npm/linter/oxlint/scripts/n8n-reference-source.mjs:54` and `packagePrefix` in
  `npm/linter/oxlint/scripts/n8n-preparation-overlap.mjs:41`, with a comment that these
  are historical paths.

**source-coverage**
- (a) vize_l2 `vue_setup_*` corpus tests, since 11-12. `davinci/vize_l2/tests/support/vue_setup.rs`
  walks the generated, gitignored `npm/ui/registry/files/**` (5563 vs 3187 sources).
  **Test bug.** Collect only git-tracked files and do not re-pin the counts.
- (b) `diagnostic_digest_hints_cli` 2 tests, since 11-18. Culprit #13248 (f5626ed856): ANSI
  colour under `vp run`. **Test bug.** Add `NO_COLOR=1` and remove
  `CLICOLOR_FORCE`/`FORCE_COLOR` in both helpers, as `check_text_diagnostics_cli.rs:71` does.

**Tooling shards (8 files)**
- **Likely real regressions**:
  - `lsp-completion-resolve`, since ≤11-12: generated helpers (`__vizeDirective`,
    `__vize_defineProps`, …) are offered as script completions.
  - `lsp-script-completion-boundary`, since ≤11-16: candidate `data` is lost and the setup
    `defineProps` snippet is missing. Probably the same root cause.
  - `lsp-call-hierarchy-vue-exports`, since ≤11-13: outgoing-call `range` is now the name only
    instead of the whole declaration. That breaks the LSP rule that `range` encloses the body.
  - All three: bisect maestro/canon commits from 11-11 to 11-13 and land `fix(lsp)` with pinned
    lsp fixtures.
- **Behaviour change, undecided**: `lsp-template-completion::component prop completion tracks
  unsaved child edits`. A new `script/define-props-destructuring` warning shows in the editor.
  Was a preset change deliberate?
- **Stale baselines**, from the per-rule docs page move (`docs/content/rules/reference/*.md`):
  `css-engine-boundary` (6 linter cells, 84 vs 90) and `lint-ssr-script-setup` (+36 B ×3). Route
  the expectations through `tests/tooling/support/current-rule-documentation.ts`; never
  hand-edit them.
- **Stale pin of a deliberate, Vue-matching change**: `differential-compiler-api-execution`.
  `vue` imports now bind as `setup-const`. Regenerate the `currentReference` successors in
  `sfc-fix-history.manifest.json` with the fixture tool, and diff the 2 rows not yet compared.
- **Test bug**: `differential-formatter-api-execution`. #12976 (df752d6395) overwrote the shared
  `crates/vize_glyph/tests/snapshots/history_template_dynamic_attribute_barriers.snap.txt`. The
  formatter output still matches the pinned sha. Give the Rust test its own snapshot file and
  restore the 154 B reference.
- The docs-render-concurrency-browser shard's real failure is `lsp-script-completion-boundary`.

Fork scratch (machine-local): scratchpad `mh/` and `mhB/`, plus `scripts/drift*.py` and
`scripts/snapdiff.py`.
