# Superseded PR runs stop without reporting

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

When a `Check` run is cancelled, it now ends in well under a minute:

- Evidence steps in the PR JS, tooling and merge-queue Rust jobs run on
  failure but no longer on cancellation (`!cancelled()` instead of `always()`).
- The `Rust source report` and `PR source report` jobs are skipped when a job
  they need was cancelled.
- `test-report` still always runs, so a cancelled run can never leave the
  required check skipped, and still fails when a job it needs was cancelled.
- `test-report-comment` does not run for a cancelled run.

Superseded merge-queue candidates are stopped by
[`cleanup-obsolete-merge-groups.yml`](./2026-10-13-6830-obsolete-merge-group-cleanup.md),
which also checks the current queue. A second watchdog that only polled each
candidate's ref was removed as a duplicate.

## Reason

A new push to a PR cancels the previous `Check` run, and the new run cannot
create its first job until the old one has finished. On 2026-10-13 that took
up to 172 seconds: cancelled jobs still ran their `always()` evidence uploads,
and then the `always()` report jobs ran one after another, each with its own
runner start and checkout. The superseded run's reports and evidence are never
read.

## Tradeoff

A run with a cancelled or timed-out job no longer uploads the test inventory
or posts the PR test report comment. The required `test-report` check fails in
both cases, as before.

## Scope

The canonical corpus observers and the native phase evidence workflow keep
their `always()` steps for now; they account for most of the remaining
cancellation latency and are changed separately.
