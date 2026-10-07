# Matrix evidence after a failed-shard retry

Tracking issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

A successful failed-job-only retry can leave two artifacts with the same shard
name. The release preflight previously required global name uniqueness, so it
rejected the successful retry even when the current shard had complete reports
and a successful surface verdict. Artifact IDs do not encode execution order.

Select evidence only from the selected successful Real Project Matrix run and
its exact source SHA. Fetch its current jobs and earlier attempt snapshots.
Authenticate each shard's run ID, source SHA, attempt, successful current job,
and complete execution timestamps. Associate an artifact with exactly one
execution through its authenticated run/source/branch metadata and creation
timestamp inside that shard execution. Unknown or overlapping ownership,
multiple current artifacts, expired current evidence, absent history, and
missing current evidence remain failures.

GitHub copies completed jobs forward with new API IDs during a failed-only
retry. Their original execution timestamps remain unchanged. Resolve those
aliases to the first authenticated execution, retain its original job ID and
attempt, and record the current observed alias separately. An old execution
must precede the selected attempt's start; a fresh execution must not.

The artifact API does not directly supply a producing job ID. The association
uses authenticated same-run metadata, the trusted source workflow's exact
shard/job naming, and an unambiguous execution time window; it does not invent
an API-provided job link. Print the complete selection provenance, including
every historical same-name artifact. Preserve old artifacts and receipts.
Never select by highest artifact ID or globally newest creation time.

The original archive validators run unchanged on the selected set. Every
shard still requires its complete reports and actual successful surface
verdict. Existing typecheck, parity, mode, budget and archive policies remain
unchanged, including the normal preflight's explicit Optional typecheck
policy. Publisher authorities, package inventories and release dispatches
remain unchanged. This is a future release improvement; the frozen first
release head and its existing operator continue independently.

The read-only replay of run 37695388101 at source
`fd6241bf8ea5466794cc955138a59a9b75a8aac2` selected all 22 shards. Shard 19's
successful attempt-2 artifact 11516467570 has a lower ID than retained failed
attempt-1 artifact 11516590462. The other 21 shards retain their authentic
attempt-1 producer IDs despite their attempt-2 API aliases. This historical
metadata replay is not a new release qualification or a published result.

Authored Rust and Node controls cover this two-attempt case, original carried
IDs, ambiguity, missing fresh evidence, foreign sources/attempts, malformed
timestamps, failed current jobs and expired evidence. Node pipeline controls
also require the selected complete archive and reject a missing surface
verdict while never reading the historical partial archive. The existing
preflight and archive controls remain intact. Fresh source Actions and actual
protected delivery remain required before this change is complete.
