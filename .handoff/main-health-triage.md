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
