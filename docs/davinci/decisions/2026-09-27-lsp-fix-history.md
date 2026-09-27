# LSP fix-history response fixtures — #6883

## Pinned scope

The issue's original 238 fixes of 421 touching commits is a historical snapshot.
At main `9aaa1fe458a09e0d0c6604dc8835ccf7c737d943`, the same narrow path scope
has 239 nonmerge subjects beginning `fix`, among 424 nonmerge touch commits
and four merges. Reproduce it
with `git log --no-merges --format='%H%x09%s' --grep='^fix' <pin> -- crates/vize_maestro`.
The [review ledger](../plan/lsp-fix-history.tsv) preserves every selected SHA and
subject. `pending-response-audit` means unassessed, not proved missing or covered.

Title matching is only the issue's initial denominator. Observable requirements
also survive feature/refactor/chore changes, editor package moves and Canon
transport changes. The earlier local expanded audit at
`9e203108c7b77b8a0bf6243d5db1f2c09c878eac` remains separate prepared evidence.
Its broader Canon transport, editor lineage and provider scope still requires
complete response reconciliation. It is not used as this ledger's denominator
or as proof of whole-history coverage.

## First small executable fixture

Fix `447716ebc8a2d52e810d2be1de6a8166a95e4c32` stopped inactive comments and
strings from becoming document links while keeping multiline imports/exports.
Its original `Host.vue` input and three dependency files are stored byte-exact
under `tests/_fixtures/differential/lsp/inactive-imports/` as `.vue.txt`, with
explicit runtime filenames and SHA256 values in `case.json`. The archive files
do not become additional Vue corpus members.

The existing production stdio regression now consumes those files. It checks the
complete ordered `textDocument/documentLink` result against
`response.expected.json`, including all ranges, targets, omitted optional fields,
array order and null-versus-array behavior. Only the whole temporary workspace
URI token is materialized, using its canonical directory. No actual result is
projected or sorted. Readiness requires a publishDiagnostics on the exact entry
URI with a present matching version 1; absent or stale versions cannot pass.
Both the fixture and actual response must contain exactly the original three
active links, so an emptied expectation cannot accept an empty server response.

The expectation is derived from the original authored string ranges and ordered
targets, independently of a new product implementation. A focused actual stdio
replay passed against the historical f59 release Actions artifact:

- Product source: `f59e69c38ecbead394ba30f0fdacb5f9c1b9fd04`.
- Actions run 36267966910, artifact 10915050776, aarch64-apple-darwin CLI.
- Actual binary SHA256: `b011b7bdba3f9b4a42a25d329a2498d80e2188f93607a8340e12c79ff7f91c68`.
- Node v24.14.0; one selected runtime test passed, then the whole document-link
  file passed eight tests, with zero failures/skips/cancellations.

This old binary validation is separate from a future fresh-source Actions run.
No build of this fixture branch, full raw-wire archive, shared-runner adapter,
native L4 result or whole-fix coverage has been established here.

## Reuse and remaining work

The first LF/CRLF observer pack (`1da2c5282a66693354c11929b5efab151140ee5e`)
and surrogate/child-prop pack (`8dd742e62b47d5767778cd7fcb8420ec83e1970b`)
remain identified as external prepared evidence. They are not copied, promoted
to passing adapters or credited as whole-fix closure by this slice.

Before #6883 closes, reconcile every observable historical requirement with
existing complete snapshots or a new fixture; run the actual required legacy
lane with fresh source/binary receipts; and register complete responses/state
transitions through the shared #6891 harness. Retain initialization, error
envelopes, diagnostic order/versions, close/reopen/cancellation, UTF-16 guards,
checker non-start and transport-generation requirements. Full native L4 parity
and provenance remain required before replacing the legacy product path.

The selected-request fixture contributes no native acceptance credit. #6883
stays open, and the ledger's remaining entries retain explicit pending states.
