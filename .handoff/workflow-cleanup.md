# Workflow cleanup handoff (2026-11-24, ~21:45 JST)

Delete this `.handoff/` directory before merging this branch.

## State

| Step                                                                    | Branch                                  | State                                                                                                                                                                                                                                              |
| ----------------------------------------------------------------------- | --------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A, B, D, E, F, G, H combined                                            | `ci/delete-one-off-workflows`           | PR #13633, auto-merge on. Its only red checks come from `canon-upstream-diagnostics.test.ts › slots`, a main regression from #13603. The fix #13636 was in the merge queue (position 4 at 21:33). Main has 260 workflows; after #13633 it has 125. |
| I1–I3 + K combined                                                      | `ci/merge-workflows-and-named-programs` | Pushed, no PR. Stacked on #13633. 125 → 107 workflows. PR body: `.handoff/workflow-cleanup/body-combined2.md`.                                                                                                                                     |
| Generated-file headers (coordinator)                                    | `ci/generated-file-headers`             | Pushed, no PR. Lands right after #13633. Its headers are regenerated on rebase.                                                                                                                                                                    |
| J0: CI library move to `tools/ci`                                       | `ci/move-ci-library` (this branch)      | Pushed, no PR. Stacked on the second combined branch.                                                                                                                                                                                              |
| J1–J4: move and rename the workflows into `tools/ci/workflows/NN-name/` | —                                       | Not started. Waits for the drain PRs (#13589, #13539, #13571, #13574, #13428, #13429, #13431 were open at 20:45).                                                                                                                                  |

Closed in favour of #13633, branches kept: #13605, #13610, #13606, #13609, #13607, #13608, #13626. Merged: #13604 (C).

## Resume steps

1. **#13633.** After #13636 merges, run `gh run rerun <run-id> --failed` on #13633's latest Check run (its only failures are the slots shard plus the reports downstream of it). Auto-merge is on, so a green rerun enters the merge queue. If main moved and the PR went DIRTY: `git merge origin/main`, then take main's `tools/config/retired-tooling-languages.json` and drop the entries of missing files (the clean-index step in `.handoff/workflow-cleanup/clean-index.py`). Then run `node tools/commands/generate/repository-config.ts`, `vp check`, the workflow tests, and push.
2. **Second combined PR**, right after #13633 merges:
   1. `git rebase origin/main` on `ci/merge-workflows-and-named-programs`. #13633 is squashed, so drop its merge commits: `git rebase --onto origin/main ci/delete-one-off-workflows`.
   2. Run the generator `--check` and `vp check`. Refresh the byte pins with `refresh-history.py content-mapper-conformance release-contracts` after `vp fmt`.
   3. Run `node tools/commands/release/v1-support-matrix.ts --check`.
   4. Push, then `gh pr create --title "ci: merge workflows and move long inline commands into named programs" --body-file .handoff/workflow-cleanup/body-combined2.md`, then `gh pr merge N --auto --squash`.
3. **Header PR** `ci/generated-file-headers`: the coordinator lands it right after #13633.
4. **J0** (this branch), after the drain finishes and the second combined PR merges:
   1. Rebase onto main, including the header PR. The PR (`ci: move the CI library to tools/ci`) is drain-gated because it edits `check.yml` and the other frozen workflows' staging paths.
   2. After a rebase, re-run the moves from the commit message if conflicts get large:
      - lib: `pkl_move.move_files` for `tools/config/pkl/{Actions,BootstrapAssets,MapperSdks,NativeChecks,RustFeatureTests,ShellScript}.pkl` and `parts/{shell,test-native-canon}` → `tools/ci/lib/`;
      - `.github/scripts/workflows/shared/*` → `tools/ci/lib/scripts/`, rewriting `../shared/` → `../../../../tools/ci/lib/scripts/`;
      - `tools/commands/ci` → `tools/ci/commands`.
   3. Do not rewrite frozen records: `tests/tooling/fixtures/**`, `tests/_fixtures/**`, `*.before.yml.txt`, `*-original-*`, or strings in successor/transition helpers. Tests that compare against them use `tests/tooling/support/ci-library-paths.ts`.
5. **J1–J4:** `python3 .handoff/workflow-cleanup/move-rename-workflows.py <map.tsv>` on top of J0. The map rows are `old<TAB>new`: `rename-map.tsv` holds all of them, and `J1-map-live.tsv` holds the 64 for J1 (groups 03–11).
   - **Split:**
     - J1: groups 03–11.
     - J2: group 01 plus level-deletion-readiness. Update the run lookups in the same PR: `cadence.ts`, `consumer_recovery/{custody,policy}.ts`, `canonical-corpus-workers.mjs` (accept the old and the new path for one release cycle), the two `.jq` files and `legacy-deletion-guard.ts`.
     - J3: the 6 group-02 renames, and the sources of the 9 protected files move to `tools/ci/workflows/<protected name>/` while their output names stay.
     - J4: composite actions move to `tools/ci/actions/` and the old-layout support leaves the generator. Drop the legacy name list in `tools/support/ci/workflow-names.ts`.
   - **Known hand work** (from the J1 dry run, 47 cases in 29 files: `J1-trial-*.txt`):
     - Re-pin the custody chains: workflow-program-history, `language-project-classification-runtime.ts` with typescript-plugin, typescript-provider-vsix and language-project-fixtures, and the native-smoke babel inventory. Keep the pins.
     - The successor chains in native-smoke and the canonical corpus.
     - The phase-3 witness ledger (`docs/davinci/plan/phase-3-records`) is live.
     - Planner keys such as `musea-browser` must not be renamed.
     - Re-run `vp fmt` on the re-wrapped lines.
   - Per-workflow `jobs/` folders are approved; only the shared part sets live in `lib/parts`.
   - `03-vue-benchmarks` is approved.
6. **Seed runs after J2:** a push run of `01-check.yml` on main, and one dispatch of `07-level-deletion-readiness`. A dry run of release cadence before its first automatic tick.
7. **L (docs):** CONTRIBUTING, the living docs and their translations, and the catalog group titles. It rides with J1/J2.

## Files in `.handoff/workflow-cleanup/`

- `move-rename-workflows.py`, `pkl_move.py`: the J move and rename. `rename-workflows.py` is the older rename-only variant.
- `rename-map.tsv`, `J1-map-live.tsv`, `J-plan.txt`, `J-notes.txt`, `J1-trial-*.txt`.
- `body-combined.md` (#13633), `body-combined2.md` (the second PR).
- Helpers:
  - `extract-runs.py`: moves long `run:` commands into named programs.
  - `orphan-drivers.py`: lists unreachable drivers.
  - `refresh-history.py`: refreshes the byte pins.
  - `closure2.py`: deletion closure via git grep.
  - `clean-index.py`, `resolve-rebase.sh.txt` (run with bash), `resolve-list-conflict.py`.
  - `kscan.ts`: the size-guard scan.
- `tools-not-covered.tsv`: the one-off tools files left to the tools sweep.

## Red kept workflows (not fixed in this effort)

e2e, davinci-resource-budgets, n8n-adoption.
