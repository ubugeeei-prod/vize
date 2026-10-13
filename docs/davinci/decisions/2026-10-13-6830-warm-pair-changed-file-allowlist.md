# The warm pair measures any production change

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

`tests/performance/support/warm-type-backed-workflow.ts` no longer keeps a list
of the production files a pull request may change. The `paired-original` job
of the Canon distinct-host scaling workflow builds the merge base and the head
and compares every original warm request for whatever the pull request
changed.

The driver also stops requiring that a harness-only change touches only a
named set of harness files. It still checks that the original request inputs
at the merge base match their recorded digests.

## Reason

The list named about 130 files under `crates/vize_canon`, `crates/vize_maestro`,
`crates/vize` and `crates/vize_patina`. A pull request that changed any other
production file failed with `unqualified production delta` before anything was
built or measured, so each such pull request had to append its own paths to
the same list, and every two of them conflicted. The failure said nothing
about the product: the comparison that follows is the check.

This follows the rule recorded for the
[star listing ledger](./2026-10-13-6830-star-listing-recorded-commit.md): a
test verifies product behaviour or an immutable past fact, not which files an
ordinary pull request edits.

## Scope

The host qualification modules that the list used to consult
(`warm-type-backed-*-host.ts` and their manifests) are no longer called by the
driver. They stay in the tree until the ledgers that still name them are
retired, which is tracked on #6830.
