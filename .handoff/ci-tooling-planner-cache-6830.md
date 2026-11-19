# Handoff: ci-tiers track (#6830), paused 2026-11-19 ~12:15 UTC

Delete `.handoff/` before this branch merges.

## This branch (`ci/tooling-planner-cache-6830`, not yet a PR)

It stacks three commits on main. The first is a copy of #13401's commit; the other two are new:

1. `ci: select tooling tests by what they read`, the same as open PR #13401.
2. `perf(ci): read each tooling planner input once per pass` memoizes file reads and stat calls for one `planToolingTests` pass. Local planner time went from 9.2 s to 5.4 s. Selection was byte-identical on 25 replayed main commits: run `.handoff/ci-tiers-6830-data/run-sample.sh` and compare against `sel-after2.json`.
3. `docs(davinci)` corrects the redesign record. The merge queue uses the PR-tier tooling planner on the merge-group diff. The full tooling suite runs only in nightly Full qualification.

Next steps:

- When #13401 merges, rebase onto main; commit 1 then drops.
- Open the PR with measurement: plan-job step time before and after, shard replan time.
- Run the planner tests: `tests/tooling/tooling-test-selection.test.ts`, `tooling-product-inputs`, `tooling-reference-accuracy` and `ci-iteration-plans` (54/54 passed locally).
- Enable auto-merge.

## Open PRs and in-flight runs

- **#13401** (tooling selection accuracy) is OPEN with auto-merge off. Merge-queue run 44509925015 **failed**: shard `Affected tooling (docs-heading-hash-browser + 53 files)`, job 135516363555. I have not yet determined whether this PR caused it. Read that job's log (the API call I used to fetch it returned 404). Then fix it, or requeue with `gh pr merge 13401 --auto` if the failure is a flake.
- Merged today: #13364, #13368, #13380, #13396 (skip jobs hosted, plus `docs/davinci/decisions/2026-11-19-6830-pr-check-redesign.md`) and #13402 (side workflows off PRs).

## Measurements

- Before: 58 successful PR Checks on 2026-11-18/19 had p50 12.3 / p90 19.3 min. Run ids are in `before-ids.txt`; jobs are in `jobs60.json.gz`.
- After: there is no fair sample yet. Only about 10 PR runs have finished since the changes, and fleet runner queueing dominated them (jobs waited 5–10 min).
- Collect ≥ 20 successful PR runs created after 2026-11-19T10:45Z:
  - `fetch-runs.mjs <out> <created-range>` lists runs;
  - `fetch-jobs.mjs <runs.tsv> <out.json> N` fetches their jobs;
  - `analyze.py <json>` computes p50/p90, last job and per-job stats.

  Post the result on #6830.

## Remaining redesign steps (record steps 3–5)

1. Single report job. The first candidate is dropping the `Affected tooling report` hop by moving its rule into `canonical-corpus-selection.mjs --check`, like `canonical-corpus`. About nine tests reference the job.
2. Prebuilt CLI/NAPI artifacts keyed by the Rust source cone (tree hashes). This needs a decision on receipt custody, which currently binds to the whole HEAD/tree.
3. Self-planning jobs with small fixed matrices.
4. Measure `incremental = true` for the PR `ci` profile on the sticky target; the Rust cohorts' build phase is about 1.5 min.

`Rust tests (Type checker)` alone takes 4.8 min, so p50 ≤ 3 min also needs the cohorts about 3× faster.

## Decisions still needed

- Receipt custody for source-hash-keyed prebuilt artifacts.
- Whether tsgo-required Type checker tests stay on PRs (#11489) given the 3-minute target.

## Data

`.handoff/ci-tiers-6830-data/` holds the analysis scripts (fetch/analyze/replay) and their inputs and outputs, copied from the session scratchpad.
