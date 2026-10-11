# Preserve native phase evidence across PR metadata edits

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The native backend workflow listens for `pull_request: edited` because changing
a PR base changes its comparison baseline. The same event also covers title and
description edits. Its original PR-number concurrency group cancels an active
native phase run for any such edit, even when the source SHA is unchanged.

For cache PR #8561 at immutable source `c4828443ddf10b6579a3024c7882da2c7671eb49`,
native phase runs [38106019320](https://github.com/ubugeeei-prod/vize/actions/runs/38106019320),
[38106022360](https://github.com/ubugeeei-prod/vize/actions/runs/38106022360) and
[38106167242](https://github.com/ubugeeei-prod/vize/actions/runs/38106167242)
were created for the same head while the PR metadata changed. The first two
were cancelled. Their exact finished conclusions must not be reused as current
acceptance. There is no measured throughput improvement yet.

## Decision

Retain `opened`, `synchronize`, `reopened` and `edited`, all existing source path
filters, manual immutable-source execution, source/base receipts and the full
native phase workload. An edited event runs that workload when its payload
contains `changes.base`. The [official pinned Octokit webhook schema](https://github.com/octokit/webhooks/blob/aa80c24a3f55354465b8b1438d33dd5602956d0d/payload-types/schema.d.ts)
represents a base change with `ref.from` and `sha.from`, separately from
`changes.body` and `changes.title`.

Metadata-only edits use a unique run-ID concurrency group and skip the job.
Their configured name selects `Native phase metadata edit ignored`, keeping
the genuine `Actual CLI shard native phase evidence` check intact. GitHub leaves
a skipped job name expression unevaluated; that literal context also remains
distinct from the genuine evidence context. A job condition alone
would still cancel genuine work at workflow concurrency admission. Reusing the
genuine check name for a skipped metadata event would also confuse current
evidence. [GitHub's concurrency documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#concurrency)
specifies that group admission can replace pending runs and that
`cancel-in-progress: true` also cancels active runs.

Source updates and real base changes keep the existing PR-number group and
cancel stale native phase work. Manual runs keep their unique run-ID group.
No checks, commands, workload assertions or validation budgets are removed.

## Verification and remaining evidence

Source-backed tests execute the actual YAML concurrency, job-name and job-if
expressions with title/body/mixed edit fixtures and the official base-change
payload shape. They require inert runs to have separate groups and check names,
prove that both running and pending genuine evidence survive, and retain all
source/new-head/manual/base-change execution. The old shared-group policy is a
counterfactual cancellation control.

Hosted source `9d58deb` metadata run [38107363044](https://github.com/ubugeeei-prod/vize/actions/runs/38107363044)
skipped while genuine native run [38107284879](https://github.com/ubugeeei-prod/vize/actions/runs/38107284879)
stayed active. That unchanged-source routing witness is historical after any
successor; all fresh exact-head Actions and protected merge qualification remain
required.
After actual merge, verify that a genuine source run stays active during a
necessary metadata edit and that a real base retarget still reruns the workload.
This event-routing fix grants no type-checker performance or accuracy credit.
