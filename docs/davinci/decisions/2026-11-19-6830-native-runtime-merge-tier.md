# Native runtime lanes run in the merge queue

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

The eight `Native runtime (…)` lanes (canon, lsp, ssr, scoped-css, setup,
vapor, attributes, selected-template) execute real Vue, Vite and tsgo oracles
against source-built products. #6830 puts tsgo-required and runtime oracle
work in T1, not in the T0 PR tier. A fast-tier PR now plans these lanes and
defers them: its matrix is `none` and the plan summary lists the deferred lanes
as `T1: …`.

[`plan-pr-checks.ts`](../../../tools/support/compat/github/plan-pr-checks.ts)
still computes the affected lanes for every event. They run on the PR when
[`select-pr-tier.ts`](../../../tools/support/compat/github/select-pr-tier.ts)
selects the heavy tier: the `full-ci` label or an unknown (empty) diff. Every
merge candidate runs exactly the lanes the same diff selects, so the lanes a PR
deferred run before anything reaches `main`.

This narrows the PR half of the
[2026-10-16 native feedback decision](./2026-10-16-6830-native-feedback.md),
which ran the explicit native families on PRs and merge candidates. The merge
candidate half is unchanged. The affected Rust cohorts, including the Type
checker cohort with tsgo required (#11489), stay on the PR.

## Why the guarantee is unchanged

The protected merge queue is the only route to `main`, and it runs these lanes
on the actual merge candidate. A PR run checks a merge ref that goes stale as
soon as `main` moves, so for these lanes it was a second execution of the same
check, as for the heavy lanes in the
[2026-10-13 decision](./2026-10-13-6830-pr-tier-heavy-lanes.md).

## Tradeoff

A lane failure that a PR used to show first now ejects the queue candidate.
Authors of native runtime fixes add `full-ci` to see the lanes on the PR.

## Measurements

Last 58 successful `pull_request` Check runs before this change (2026-11-18 to
2026-11-19, run ids in the PR): p50 12.3 min, p90 19.3 min. `Native runtime
(canon)` ran in 51 of them for 7.8 min at the median (10.3 min p90) and was the
last product job to finish in 31 of the 58 runs. On 2026-10-19 the same lane
took 2.2 min. A PR run used 32 to 75 Blacksmith jobs against a pool of about
40 concurrent jobs; the eight lanes are eight of them.
