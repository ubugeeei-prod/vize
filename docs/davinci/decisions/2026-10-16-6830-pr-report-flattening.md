# Validate PR results in one final report

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

At `59983fff77ac67ea8d062003d24e032c0a5b3f4d`, all product jobs in
[Check 38924772982](https://github.com/ubugeeei-prod/vize/actions/runs/38924772982)
passed. Compiler was the last product job, completing at 15:37:12 UTC after
252 seconds. Three serial report jobs then added 124 seconds:

| Report                 | Created to actual start | Execution |
| ---------------------- | ----------------------: | --------: |
| Rust source report     |                    46 s |       8 s |
| PR source report       |                    36 s |       9 s |
| Required `test-report` |                    17 s |       8 s |

These are terminal Actions API `created_at`, `started_at` and `completed_at`
timestamps. Each report was created when its last predecessor completed, to
the API's one-second precision. The 99 seconds before actual starts are
observed waits; they do not establish a runner-capacity or provider cause.

## Preserve the gates without two PR runner allocations

The two reusable workflows export their actual `jobs` contexts. GitHub
[documents job results and outputs in reusable workflow outputs](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#jobs-context)
and [forwarding those outputs to callers](https://docs.github.com/en/actions/how-tos/reuse-automations/reuse-workflows#using-outputs-from-a-reusable-workflow).
The final required job receives the source results, including the Rust
workflow's nested raw results and mandatory `run-rust` input. Neither workflow
call is itself a matrix call; the Rust matrix result retains the same aggregate
semantics used by its previous report. No per-worker verdict is inferred from
a last-completed matrix output.

On PRs, `require-pr-checks.ts` invokes the existing `requireRustTier` and
`canonicalSourceReport` policies in the existing `test-report` job. It requires
every named source dependency and both required Rust jobs. Missing, blank or
malformed outputs fail. Failed, cancelled and unexpectedly skipped jobs fail,
even if a parent reusable-workflow result says success. The canonical corpus
selection and the two existing queue-only qualifications retain their skip
policy. Only the now queue-only source report is removed from the PR packet;
unknown additional source jobs remain gated.

The existing final aggregate step still validates every top-level dependency.
Its name, dependencies, permissions and `always()` condition are unchanged,
including compatibility with the stacked instruction-count report policy.
No API poller, receipt cache, test filter, timeout, concurrency, runner or
product command changes are introduced.

Both intermediate reports continue to run in the merge queue. Their steps,
including exact-run worker selection, source identity, artifact digest and
observation aggregation, are unchanged. Cancellation still cannot produce a
successful required final report.

## Validation and measurement boundary

All 100 distinct focused tests passed. The new controls exercise ordinary and opted-in PRs, every missing source
job, failed/cancelled/skipped Rust leaves behind successful rollups, disabled
Rust, malformed output packets, unknown extra source jobs and the real Node
CLI. Existing source, Rust, queue and canonical-selection contracts also run.
Actionlint 1.7.12 accepts the workflow outputs and nested forwarding. Scoped
formatting, lint and strict native TypeScript checks passed. An independent
comparison against the prior source confirmed unchanged queue report steps,
dependencies, permissions and runners.

Removing the first two observed report stages models 99 seconds saved:
407 seconds becomes 308 seconds if the last report's observed 25-second wait
and execution remain unchanged. This is a replay, not a hosted measurement,
and still exceeds five minutes. A fresh exact-head PR run must establish
actual output delivery, strict final success and end-to-end latency.
