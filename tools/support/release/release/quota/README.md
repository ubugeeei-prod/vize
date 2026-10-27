# Rolling publication admission

The budget is an internal safety policy: **19 versions after a proposed crate
write**, leaving one below the 20-version ceiling observed during release
recovery. It is not a statement of crates.io's guaranteed quota and does not
reserve a registry slot.

Every check uses the publisher's canonical crate inventory. It enumerates the
entire `sort=date` history through validated same-origin seek cursors, up to ten
pages of 100 rows. An old row on the first page never establishes coverage.
Totals must remain constant, every numeric version ID and version must be
unique, all crate identities and timestamps must be valid, and creation order
must be nonincreasing. All versions count, including prereleases and yanked
versions, in the inclusive 24-hour window plus a two-minute margin.

Requests have a 15-second timeout and a 2 MiB response limit. The complete
decision expires after two minutes. Response Date must agree with the local
clock within two minutes; Age, when present, must be a nonnegative integer at
most 60 seconds. Missing Age is retained as missing, rather than reported as
zero. Cache-busting requests and the prior-version witness provide additional
freshness checks. HTTP failures, 429, malformed data, missing pages, cursor
loops, changed totals and uncertain clocks refuse admission.

The prior release's existing npm completeness policy is unchanged. If a prior
crate version is absent from the complete history, only a fresh exact-version
HTTP 404 establishes that absence. A positive exact-version response with a
missing history entry is inconsistent and refuses admission.

## Admission phases

Cadence and the manual cut use the same helper. Preparation checks before any
version stamp. The Release workflow then checks again after its long CLI,
native, WASM and package-smoke builds, before any initial registry write.
Every initial registry publisher depends on this shared admission. The crate
publisher also checks immediately before writes, including a failed-job retry.
The existing green-main, source, serialization and completeness checks remain
in place.

A recovery version is not identified by its version number alone. The genuine
original Release run, its workflow head, the requested producer source, and the
immutable prepared tag commit remain separate identities. The tag and stamped
native manifest must match; the producer must be an ancestor of the original
main workflow head. Existing crate rows require that original run's trusted
publication provenance and a matching checksum/identity from the independent
exact-version endpoint.

A verified existing own crate consumes zero new writes. Consequently, all
already-published own crates can finish npm/editor/JSR channels even when their
window count exceeds the new-write budget. Missing crate targets must still
pass the budget. A same-number foreign version or a checksum mismatch never
qualifies as recovery.

## Qualification and operation

The read-only `quota_dry_run` input on `release-verify.yml` runs the mock
admission controls and reads live histories. It neither stamps nor publishes;
the ordinary npm verification and dependent JSR publication jobs are skipped.
The target version is an admission calculation, not a claim that it exists.

Receipts retain request URLs, status, Date/Age, full-history counts, page byte
counts and SHA-256 hashes, recent timestamps, producer identity, recovery
proof hashes and the next conservative eligibility time. Actions artifacts
retain them for 90 days. Unavailable state has no invented eligibility time.

A registry refusal is not permission to cut another version. Recover the same
immutable cut. Cadence must only be enabled after the guard's exact-head
qualification, normal checks, protected merge and operational review. The
existing ten-minute interval is a burst limit; this rolling guard supplies
the publication budget.
