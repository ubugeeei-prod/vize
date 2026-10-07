# Protected observer parallelism: local proposal

Status: unpublished local implementation for root review. No PR, remote branch,
Actions dispatch, queue mutation, or release operation belongs to this change.
Actual throughput improvement remains **unmeasured** until exact-head Actions.
The delivery policy in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6037135723)
continues to govern finite cohort admission and verified release completion.

## Evidence and decision

The successful protected [Check 37612234749](https://github.com/ubugeeei-prod/vize/actions/runs/37612234749)
on `538370a5db8f4443538145c8ae891e5e84008328` took 20m26s from run creation
to final update. Its canonical job took 19m22s: fixture hydration 75s,
DOM 52s, SSR plus Pug 401s, and production reach 595s. These are historical
timestamps from actual jobs and raw logs, not measurements of this proposal.
The Rust producer took 12m41s, including 483s of feature differential work after
its completed 4,236,383,020-byte archive upload. Workers waited 507.703s after
archive completion for that dependency. A timestamp replay with only canonical
parallelism and unchanged Rust gives about 17m12s; it proves no new runtime.

Schedule DOM, SSR plus Pug, and production reach as three independent required
workers. They retain the complete original commands, fixture exclusions, runtime
setup, features, harnesses, byte comparisons, native reach controls, and budgets.
One finalizer retains the existing `s2 dom corpus` job name and legacy evidence
artifact. It succeeds only after all three workers succeed and their actual
GitHub executions and artifacts pass custody checks. No compiler or level gains
a pipeline stage or serialization; only CI job dependencies change.

Move the original merge-group Rust differential composite and fixture coverage
step into a sibling job alongside archive workers. The sibling depends on the
completed archive producer, verifies its unchanged HEAD/tree/archive digest and
runtime receipt, then restores the exact archive paths before running the same
recipe. The Rust report requires producer, every shard, and differential sibling.
An empty, skipped, failed, cancelled, or pending required tier fails closed.
Ordinary PR producer/shard behavior and worker selection remain unchanged.

## Complete fixture ownership and hydration

Cache only submodule Git object directories, keyed by committed `.gitmodules`
bytes and the entire ordered 147-path gitlink vector. PR and merge-group events
restore only; trusted main events may save through the existing cache trust
policy. Every worker checks candidate HEAD/tree, index gitlinks, committed URLs,
each fixture HEAD, tracked-content cleanliness, and a full 42,998-file manifest
of relative path, whole-byte SHA-256, and byte length. Enumeration/read errors
and directory cycles fail. The same manifest is required after execution and
across all three workers. Cached test conclusions receive no credit.

[Matrix 37617991549](https://github.com/ubugeeei-prod/vize/actions/runs/37617991549)
on `8a9110229414769be71595d7888c6cbd8f3add88` observed 37,487 Vue files after a
bulk-shallow failure and successful serial fallback. Its 147 selected gitlinks
and HEAD status bytes matched the earlier complete 42,998-file Matrix exactly.
The missing 5,511 files removed seven known-invalid inputs. The original finalizer
correctly refused nine skips against the unchanged required sixteen; source
compiler/parser/corpus files were identical. HEAD-only inventory cannot establish
complete checkout. The precise historical filesystem loss cause is unavailable
because the original helper discarded underlying Git stderr.

Keep the original hydration helper and fallback controls byte identical. After it
returns success, force checkout all selected committed submodules before taking
the full-byte snapshot. Retain raw stdout/stderr, exact command arguments, source
identity, and both checkout and status outcomes in the worker artifact. The new
operation cannot recover discarded stderr inside the unchanged original helper;
its top-level warnings/stdout/stderr are retained through fail-closed pipefail
logging, including when it fails before the forced stage. Failure remains a
gate refusal with diagnostic evidence. A real temporary Git/submodule fixture
proves that a missing authored file with unchanged HEAD is restored byte exactly;
failure and replaced-diagnostic controls receive no credit.

## Execution and artifact custody

The finalizer reads official job and artifact APIs for the same repository,
caller, source SHA, run, and attempt. Required commands, snapshot, receipt, and
upload steps must finish successfully in order within their execution windows.
An older green job cannot hide a latest failure or incomplete job. A failed-only
retry may carry an earlier successful execution only when the complete official
execution signature is identical, retaining its original attempt-bound artifact.
All three artifact bodies must agree on candidate/tree, complete committed
gitlinks, metadata, whole file manifest, original sixteen skip reasons, full
canonical counters, and zero divergence/refusal requirements. The finalizer
reselects custody after downloading the exact artifact IDs with digest refusal.
For pull requests, the verified event PR head is the API provider identity while
`GITHUB_SHA` and its tree remain the candidate merge checkout identity. Both are
retained and checked separately; an API head is never credited as candidate bytes.
The verified event and official run also retain separate base and head repository
IDs, allowing a fork PR without crediting foreign base or artifact metadata.

## Review and publication TODOs

- Local focused tests, syntax/format/correctness lint, capped-file inventory, and
  frozen-source preservation are review evidence; they do not prove full corpus
  execution, Actions compatibility, or throughput.
- Root reviews this concrete implementation before any remote/public action.
  Existing finite cohorts and release recovery retain priority.
- On future admission, pair the issue comment below with a bounded reference in
  the shared decision record in the same change. Its current 350-line canonical
  layout and finite-five ownership remain untouched in this local proposal.
- Attend fresh exact-head source and full protected merge-group Actions. Compare
  actual critical-path timestamps, complete file/counter receipts, every Rust
  result, and fresh merge identity before claiming a throughput improvement.

## Paired #6830 issue-comment draft (not posted)

Protected CI can schedule the unchanged DOM, SSR/Pug, and production-reach
observers independently, then require one custody-checking finalizer under the
existing status name. All three must cover the same committed 147 submodules and
42,998 whole-byte Vue inputs before and after execution; the original sixteen
known-invalid inputs and all refusal/byte/budget gates remain required. Git-object
caches are identity-keyed and PR/merge-group restore-only. A forced checkout with
retained diagnostics closes HEAD-only incomplete-hydration credit. The original
Rust feature recipe and fixture coverage run as a required sibling after the
verified archive producer, alongside all required archive shards. No compiler
stage, serialization, budget increase, test waiver, or release acceptance changes.
Historical protected Check took 20m26s; this proposal has no Actions runtime
measurement yet. Publication remains held for root review and backlog drain.
