# LSP retained-session source associations — #6883

Paired with the [#6883 decision comment](https://github.com/ubugeeei-prod/vize/issues/6883#issuecomment-5931759221).

## Decision

Associate passive observations with a bounded pinned source catalog. The catalog
has eight response-fix candidates and two explicitly labelled safety-control
opportunities. Eight original/current whole test-file pairs are byte-identical.
Folding and inactive links retain authored input bytes, with documented changes
to expected style regions and diagnostic readiness. No entry is accepted coverage.

The [catalog](../plan/lsp-retained-session-candidates.tsv) records exact original
fix commits, original/current test-file hashes, selected current callback line
ranges, authored input hashes, local setup/options/requests, expected distinctions,
supersessions and unresolved shared helpers/dependencies. Original Git blobs and
the `cc87bb596` source pin were checked when generating it. Its bytes are pinned
by the observer's existing shared immutable-artifact reader.

## Executable association

The existing-session observer consumes this catalog at source-built launch time.
It attaches a historical candidate only when an actual caller frame matches the
catalog's current test path, complete file hash and selected test callback range.
Another test in the same file does not gain an association. A changed source hash
does not match. Corrupt pinned catalog bytes fail before a server launch; a source
tree with no catalog remains explicitly unassociated.

The actual raw process observation retains these associations alongside its
complete transport, source/build receipts and process status. Both initial
diagnostic fix associations reuse the same existing sessions; no extra launch,
version probe, build stage, workflow or test deferral is added.

Source presence at a historical commit does not prove that a test was added by
that fix or that it covers the whole change. The two safety-control opportunities
preserve valid document-color and selection responses, while their invalid-span,
Unicode and racing-state guard requirements remain unresolved. They receive their
own purpose label. Tuple `null` placeholders in the catalog's descriptions retain
the explicit JavaScript-undefined notation; they are not wire-response references.

## Remaining admission work

Every association remains pending original-input/session reconciliation. Complete
expected responses are not frozen, dependency closure is not proved, and native
handled/equivalent and whole fixes closed stay zero. Existing partial test
assertions and exact source hashes provide lookup evidence; they do not replace
whole response or full state comparisons.

The next admission slice must match actual captures with original input/dependency
bytes and request semantics, resolve helper/provider and lockfile differences,
review superseded contracts, and register immutable complete expected responses
through the shared v1 differential runner. The complete 255-SHA ledger and the
non-fix/shared-provider history still require reconciliation. #6883 stays open.

## Verification

Four source-association laws pass, covering exact current source hashes, selected
callback filtering, source/hash mismatches, shared sessions, forged positive
credits and corrupt/missing catalogs. Four existing observer laws still pass.
Strict TypeScript, selected formatting/lint and diff checks pass. This validation
is source inspection and synthetic observer evidence. Production captures,
exact-head Actions, protected merge queue and actual merge remain unverified.
