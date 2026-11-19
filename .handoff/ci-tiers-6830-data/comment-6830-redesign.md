Redesign progress after the maintainer's 2026-11-19 decisions. The design record is `docs/davinci/decisions/2026-11-19-6830-pr-check-redesign.md`.

| step | PR | state | measured effect |
|---|---|---|---|
| 1. Skip-only jobs take no Blacksmith slot (planner-conditional `runs-on`) | #13396 | merged | `Native runtime (none)`/playground skip: queue 9.2/8.9 min → 2/3 s on its exact-head run |
| 1b. Heaviest side workflows off PRs (8 workflows, 456 PR runs / 3,787 wall min per ~day) | #13402 | in merge queue | each moved workflow keeps a merge-queue, main or nightly run; pinned by tests |
| 2. Tooling selection accuracy (`node_modules`, config data, shared-helper globs; release audit) | #13401 | in merge queue | 25-commit replay: 10,349 → 9,462 selected tests, 263 → 243 shards |
| 2b. Planner reads each input once per pass | (stacked on #13401) | opens after #13401 merges | planner 9.2 s → 5.4 s locally, byte-identical selection on 25 commits |

**Correction to the design record:** the merge queue uses the PR-tier tooling planner on the merge-group diff. The full tooling suite (`--tier merge`) runs in nightly Full qualification, so tooling tests that a PR deselects still run nightly. The record says "merge queue"; a follow-up fixes the wording.

**What is left on the redesign, and why 3 minutes is still far away.** With no queueing, a typical Rust PR now has:

- plan 0.7 min, plus runner pickup about 1.0 min;
- `Rust tests (Type checker)` at 4.8 min, measured on #13368's exact-head run;
- the report chain at about 0.6 min.

That is about 7 min. Each Rust cohort and the JS builders job is 3–5 min by itself (build about 1.5 min, then tests), so a 3-minute p50 needs:

1. Self-planning jobs and a single report job: −1.3 min of fixed overhead (steps 3 and 5 in the record).
2. Rust cohort builds about 3× faster. Options:
   - `incremental = true` for the PR `ci` profile on the sticky target, which is now `incremental = false`;
   - prebuilt nextest archives keyed by source hash for the crates a PR does not touch.

   Either needs a measured CI experiment, which waits until the queue drains (>100 runs queued right now).
3. Prebuilt CLI/NAPI artifacts keyed by source hash (step 4). These help only PRs that leave the Rust cone untouched. They also need the existing receipt custody, which binds artifacts to the whole checkout HEAD/tree, to bind to the cone hash instead. That custody change needs maintainer review.

The ≥20-run after-sample goes here once the queue drains and enough PRs have run on these changes.
