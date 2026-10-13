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

A new `merge-queue-watchdog.yml` workflow does the same for merge-queue
candidates. Rebuilding or removing a candidate deletes its
`gh-readonly-queue` ref, but its runs kept going and held runners the live
candidates needed. The watchdog polls the ref every 30 seconds and, after two
consecutive 404 responses, cancels the candidate's other runs. Any other
response keeps them. It exits when those runs finish.

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
