#6830 `ci.tiers` status on 2026-11-19: what regressed, what is landing, and what needs a maintainer decision.

## Regression since 2026-10-19 (measured)

Successful `pull_request` Check runs, timed from workflow creation to the last job's completion with the jobs API:

| window | runs | p50 | p90 |
|---|---|---|---|
| 2026-10-19 | 224 | 5.7 min | 8.2 min |
| 2026-10-25 | 303 | 5.4 min | 6.7 min |
| 2026-10-31 | 140 | 11.5 min | 13.4 min |
| 2026-11-18 → 19 (58 runs, job-level sample) | 58 | 12.3 min | 19.3 min |

The step on 2026-10-31 is work, not only queueing. In 10 sampled runs on 2026-10-19 against the 58 runs now:

| job | 2026-10-19 run p50 | now run p50 |
|---|---|---|
| `Native runtime (canon)` | 2.2 min | 7.8 min |
| `Rust tests (Type checker)` | 2.5 min | 6.4 min |
| `Affected tooling (…)` shards | did not exist | 2.4 min each; 5–32 shards per PR (p50 17) |
| `PR JS package tests (builders)` | 2.7 min | 4.4 min |

Causes:

- **Canon lane.** About 30 commits between 10-21 and 11-09 each appended an exact native law to `test-native-canon`. One law, `empty_vue_star_exports::configured_native_refuses_erasure…`, takes 178 s, and the Type checker cohort runs it again.
- **Type checker cohort.** It moved to the full profile with tsgo required (#11489, 11-04). Two `empty_vue_star_exports` laws took about 190 s each. Every fix layer added to that test's inverse chain makes it slower.
- **Tooling.** PR-tier affected tooling (#12651 and others) selects 200–880 of 1,923 files, even for 2–7 changed paths, because shared helpers name `crates`, `tools/**` and glob inputs.
- **Pool pressure.** One PR run used 32–75 Blacksmith jobs against a pool of about 40, so shards were created at 2.4 min but the last one started at 5.3 min (p50).

Critical path (last product job to finish, 58 runs): canon lane 31, tooling shard 18, Type checker 4, JS builders 3. Plan job 0.7 min → runner pickup ~1.0 min → product job → report chain 0.75 min.

## Landing

- #13364 (merged 04:53 UTC): fast-tier PRs defer the 8 native runtime lanes to the merge queue. The queue runs the same lanes from the same diff, and a test pins that. `full-ci` keeps them on the PR. In 80 failed PR runs, no failure was unique to a native lane; all 7 canon failures coincided with a Type checker failure.
- #13368 memoizes the `empty_vue_star_exports` record chain (test code only). On its exact-head run, the `Rust tests (Type checker)` job took 4m 51s instead of 6m 37s. The nextest phase took 141.5 s instead of 238.6 s, with the same 4717 tests. The two laws went from 187 s and 193 s to 15 s and 27 s.

## Decisions needed (expected gains from replaying the 58-run sample)

1. **Tooling T0 selection.** Without tooling shards on the PR, the replayed p50 would be 7.9 min instead of 10.8 (−2.9 min). Tooling shards were the most common failing job in failed PR runs (55/80), so deferring broad prose/product-reference selections to T1 trades PR minutes for merge-queue ejections. Options:
   - (a) audit `tests/tooling/release/**`, which is still on the broad `tooling` scope pending its owners' audit;
   - (b) ignore glob and top-level references in shared config modules (`task-inputs.ts`, `task-commands.ts`) and in `node_modules` sources;
   - (c) move product/prose selections to T1.
2. **Blacksmith concurrency (~40 jobs).** p90 is mostly runner queueing under fleet load. Raising the cap, or running fewer path-scoped side workflows on PRs (a Canon test change triggers about 8 extra workflows), helps p90 directly.
3. **The 3-minute p50 target.** With a plan job, runner pickup, a cargo build and the report chain, the floor of the current architecture is about 4 minutes. Reaching 3 minutes needs self-planning jobs, prebuilt native/CLI artifacts per main commit and a single report job: a redesign, not a tuning pass.


The after-sample (≥ 20 product PR runs on the new planner) will follow once enough PRs have run. I am keeping this issue open; the gate is not met.
