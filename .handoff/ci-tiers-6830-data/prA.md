#6830 puts tsgo-required and runtime oracle work in T1 (merge queue), not in the T0 PR tier. The eight `Native runtime (…)` lanes run real Vue, Vite and tsgo oracles. On fast-tier PRs they are now planned and **deferred**: the matrix is `none` and the plan summary lists `(T1: …)`.

- **Merge queue is unchanged.** It plans exactly the lanes the same diff selects. A new test pins that the queue's lanes equal the PR's deferred lanes, so nothing a PR defers can reach `main` unchecked.
- **`full-ci` label or an unknown (empty) diff** (heavy tier) still runs the lanes on the PR.
- The Type checker Rust cohort, which keeps tsgo required on PRs (#11489), is **not** changed.
- Required checks are unchanged. The lane job still runs as `none` and reports success to `PR source report`.
- The decision record is `docs/davinci/decisions/2026-11-19-6830-native-runtime-merge-tier.md`. It narrows the PR half of the 2026-10-16 native-feedback decision.

## Why (measured)

**Last 58 successful `pull_request` Check runs** (created 2026-11-18 04:44 to 2026-11-19 03:16 UTC), taken from the jobs API and timed from workflow creation to the last job's completion:

| | p50 | p90 | min |
|---|---|---|---|
| Check wall | 12.3 min | 19.3 min | 5.6 min |
| `Native runtime (canon)` job (51 runs) | 7.8 min | 10.3 min | |
| Same lane, 10 runs on 2026-10-19 | 2.2 min | 2.6 min | |

- `Native runtime (canon)` was the **last product job to finish in 31/58 runs**. The next ones were affected tooling shards (18) and `Rust tests (Type checker)` (4).
- Inside the lane, one test (`empty_vue_star_exports::configured_native_refuses_erasure_at_unproved_source_boundaries`) took 178 s in job 135312670641. The Type checker cohort on the same PR also ran it, for 193 s.
- A PR run used 32–75 Blacksmith jobs, against a pool of about 40 concurrent jobs. The 8 native lanes were 8 of them.
- **Failure signal:** in the last 80 failed PR Check runs (2026-11-16 17:33 to 2026-11-19 02:27 UTC), no run failed only in native lanes. All 7 canon-lane failures coincided with a failing `Rust tests (Type checker)` (runs 44316692517, 44313136443, 44252735654, 44245970978, 44058814278, 44042184755, 44011293799). The PR copy added no unique failure signal.

**Expected effect:** on PRs that touch Canon's dependency cone, the critical path moves from the canon lane (end p50 10.0 min) to the Type checker cohort (end p50 8.6 min). PRs that never touched canon see the 7 other lanes leave their run. A follow-up PR targets the Type checker cohort's two ~190 s `empty_vue_star_exports` tests.

<details><summary>Before: run ids (total minutes)</summary>

44450333316 (16.6), 44446089283 (13.8), 44446030046 (17.3), 44442915893 (11.2), 44442173729 (7.8), 44436997534 (5.7), 44426943072 (12.8), 44426826437 (14.2), 44424149495 (16.5), 44408807230 (18.6), 44404136079 (16.7), 44401170872 (10.6), 44374872504 (11.2), 44341869903 (48.9), 44340799457 (70.2), 44337693367 (24.0), 44333367977 (24.1), 44324815143 (14.4), 44322667485 (19.4), 44317611778 (8.3), 44315884997 (10.0), 44309468539 (9.7), 44301477076 (10.8), 44297035566 (14.6), 44293712949 (15.4), 44293069055 (18.0), 44292562334 (19.2), 44290885215 (18.9), 44290136092 (9.8), 44289934057 (10.3), 44284827419 (9.0), 44282515262 (10.7), 44271857506 (16.1), 44271039903 (15.0), 44268477367 (16.3), 44268301071 (23.8), 44266505415 (14.4), 44266432334 (15.7), 44262051304 (9.4), 44260905204 (8.6), 44259922392 (14.4), 44254929843 (9.2), 44254211217 (8.5), 44252512245 (12.4), 44252432680 (10.5), 44250714739 (9.9), 44250716202 (12.1), 44250539313 (8.8), 44246261663 (8.5), 44244585429 (12.6), 44242509280 (8.5), 44242131382 (9.1), 44241756225 (8.9), 44241296661 (9.0), 44240462953 (5.6), 44240115573 (16.5), 44239724518 (8.5), 44239364581 (8.6)
</details>

## After

Exact-head Check [44459765312](https://github.com/ubugeeei-prod/vize/actions/runs/44459765312) succeeded in **5.6 min**, from workflow creation to the last job. The plan was `native: none`, and 9 tooling shards ran, the last ending at 4.6 min. This PR changes tooling only, so its run is not a typical product PR. The ≥20-run after-sample of product PRs will go on #6830 once enough PRs have run on the new planner.

## Tests

- `node --test` on the 10 planner/workflow test files that import `plan-pr-checks.ts`: 64/64 pass, including updated `pr-required-rust.test.ts` (PR defers 8 lanes; queue runs the same 8; `full-ci` runs 8) and `native-selected-setup-sfc-dom-capture-workflow.test.ts`.

## Product binaries

Unchanged. The diff touches one planner script, two tests and one decision record. No file under `crates/`, `davinci/`, `npm/`, `Cargo.*` or `pnpm-*` changes, so the `vize` CLI, NAPI and WASM outputs are byte-identical by construction.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
