# PR Check redesign for the three-minute gate

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) (`ci.tiers`: PR `Check`
p50 ≤ 3 min, p90 ≤ 6 min). The maintainer kept that target and the Blacksmith pool size
(about 40 concurrent jobs) on 2026-11-19, and asked for fewer jobs per PR plus a redesign.

## Where a PR run spends its time

These numbers come from 58 successful `pull_request` runs on 2026-11-18/19, timed from
workflow creation to the last job:

| phase                                                    | p50      |
| -------------------------------------------------------- | -------- |
| plan jobs (`iteration-plan`, `pr-source-plan`)           | 0.7 min  |
| plan done → product job running (runner pickup)          | ~1.0 min |
| longest product job (build + test)                       | 4–8 min  |
| report chain (Rust report → source report → test-report) | 0.75 min |

One run used 32–75 Blacksmith jobs, more than the whole pool, so jobs also waited for
runners. Under fleet load that wait reached 7–10 min (p50), and it sets p90. The gate needs
both fewer jobs and a shorter critical path.

## Design

1. **Jobs that cannot do work do not take a build runner.** A job whose plan selects only
   a skip notice runs on hosted Ubuntu (a planner-conditional `runs-on`). Path-scoped side
   workflows that duplicate merge-queue or nightly coverage stop running on PRs.
2. **Accurate tooling selection.** A test is selected only by inputs it actually reads.
   Path strings inside shared config modules and `node_modules` sources do not select
   their importers. Release contracts get audited inputs instead of "any input". The merge
   queue plans tooling with the same PR-tier selector on the merge-group diff; nightly Full
   qualification (`--tier merge`) still runs every tooling test.
3. **One report job.** The three-hop chain collapses into `test-report`, which needs every
   product job directly. The reusable-workflow boundaries that force the chain go away, and
   the gate logic (`require-needs-success.ts`, canonical-corpus and Rust tier rules) runs
   once over the combined `needs`.
4. **Prebuilt CLI and NAPI artifacts keyed by source hash.** The source CLI (ci profile)
   and the native addon depend on a known cone: Rust sources, `Cargo.lock`, the
   toolchain and the build profile. Trusted `main` and merge-queue runs save them to the
   Actions cache under `sha256(cone)`. A PR whose cone matches a saved key restores them
   instead of building, in the tooling shards, the JS package jobs and the playground. A PR
   that changes the cone builds once in a producer job, as today. The consumer verifies the
   restored bytes against the key's receipt, so no build from another source can be used.
5. **Jobs plan for themselves.** Product jobs stop `needs`-ing a plan job. Each one runs
   its own selector, the same pure function of the base/head trees, at its start, and
   exits early when unselected. Matrices that used to be planned (Rust cohorts, tooling
   shards) become small fixed matrices whose entries select their own slice. This takes
   the plan job and one runner pickup off the critical path.

Item 5 conflicts with item 1 for fixed matrix entries that turn out to be unselected,
because they still take a runner. The fixed matrices therefore stay small: up to 4 Rust
cohorts and 4 tooling shards on PRs. The merge queue keeps its full matrices.

## Guarantees

- Every check that leaves the PR tier keeps running in the merge queue (T1) or nightly
  (T2). Each change carries a test that the queue still selects what the PR stopped
  selecting.
- The required contexts (`check-js`, `check-vize-apps`, `fmt-rust`, `test-report`) do not
  change. The assertion lint, instruction ceilings and differential gates are not touched.
- Product binaries are unchanged: these changes affect CI only.

## Order and measurement

Changes land in this order, each in its own PR with before/after exact-head `Check`
timings: 1 → 2 → 3 → 4 → 5. After each one lands, a ≥ 20-run PR sample goes on #6830, and
#6830 closes when p50 ≤ 3 min and p90 ≤ 6 min hold over such a sample.
