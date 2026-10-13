# PR checks leave the heavy lanes to the merge queue

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

Three lanes ran on every eligible PR and again in the merge queue: the
canonical corpus observers, the Oxlint original project qualification and the
playground browser tests. The merge queue still always runs them. A PR runs
them only when
[`select-pr-tier.ts`](../../../tools/support/compat/github/select-pr-tier.ts)
selects the heavy tier:

- the PR carries the `full-ci` label, or
- it changes a path that can break those lanes: anything under `crates/`,
  `davinci/` or `vendor/`, the Oxlint, native, builder and CLI packages, the
  playground, the Cargo and pnpm manifests and lockfiles, the workflows and
  local actions, the GitHub planning tooling, and the shared fixtures.

A PR with the `hotfix` label takes the fast tier whatever it changes. It keeps
every check: the queue it jumps runs all of them on the merge candidate.

## Why the guarantee is unchanged

Nothing reaches `main` unless the merge queue passes every lane on the actual
merge candidate. A PR run checks a merge ref that is stale as soon as `main`
moves, so it was never what protected `main`; for these lanes it was a second
execution of the same check.

## Tradeoff

A lane that a fast-tier PR skipped can fail first in the queue. That ejects
the candidate and makes later candidates rebuild, so the automatic selection
errs towards heavy and only prose, tests, tooling outside the planner, and the
remaining packages take the fast tier. `full-ci` runs everything on the PR.

## Measurements

On 2026-10-13 a PR push reported its required checks after 17.1 minutes at
the median. The three lanes took 12.4 (corpus `reach`), 9.0 and 5.5 minutes
and were the three longest jobs on the PR critical path. The target is five
minutes for a PR and ten for a queue candidate.
