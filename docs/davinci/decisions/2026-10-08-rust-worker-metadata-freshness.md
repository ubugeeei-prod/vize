# Rust worker metadata freshness

Owning issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Observed failure

The protected #8153 candidate `36f3` reported an incomplete test step after all
four Rust jobs had completed successfully. The #8182 candidate `19030` later
reported an incomplete upload step with the same completed-success job state.
The collector correctly refused both packets; a single stale Jobs API snapshot
prevented it from observing the final step metadata.

## Decision

Read at most six complete paginated jobs and artifact snapshots, waiting two
seconds between reads, only when a completed-success worker has a required step
with queued or in-progress status, null outcome and null completion timestamp.
Validate every worker, required step, artifact and workflow identity before
classifying a snapshot as pending. Terminal failures and invalid or missing
packets remain immediate refusals.

Keep repository, exact source, workflow run, attempt, latest worker IDs and
complete selected artifact identities stable throughout the reads. Read the run
before and after each inventory, and rerun the original strict selector before
publishing a receipt. Pending metadata never becomes a successful receipt and
never authorizes borrowing an older artifact or an incomplete job.

The historical logs established stale step status; they did not preserve the
complete null-outcome and null-timestamp tuple. That tuple is a conservative
synthetic test control. A fresh protected candidate must qualify the real API
behavior. This change preserves workflow commands, features, inventories,
artifact checks, instruction budgets and the existing receipt schema.

## Validation and remaining work

The original selection and refusal tests plus seven new metadata tests cover
fresh recovery, six-read exhaustion, pagination, later terminal failures, foreign
or missing artifacts, incomplete jobs, and identity or timestamp changes.
Fresh exact-head Actions and protected merge-queue acceptance are still required.
Parallel canonical observers remain a separate dependent change; their current
physical fixture-cycle failure does not block this collector repair.
