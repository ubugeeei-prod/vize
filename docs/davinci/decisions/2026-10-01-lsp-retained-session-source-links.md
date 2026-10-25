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

## Current-source successor after #10437

The original catalog and its `cc87bb596` source checkpoint remain frozen with
SHA256 `0bc776ff5fc1e1d7bde32bfae865b1b87e8c84e99a6bcf5c8e2cfb4652daf4ba`.
The [current-source successor](../plan/lsp-retained-session-current-candidates.tsv)
records source checkpoint `284c7c4f6a7a1a50d9acec8d877c47aae21cfe42` and preserves
every original, authored-input, setup, request, response-description and credit
field. Only the document-color control's complete current-file hash and its two
selected callback ranges change. The observer records the current checkpoint and
the separate original catalog identity; neither checkpoint identifies a binary
executed by this source-only check.

Commit `5eeb0d7b0e37c8f0545c9f40fdf9ea3d1f2c409b` (#10437) parameterized
`withDocument` with the same default `SOURCE` and added a dense-color performance
regression case. The existing two callbacks and all their expectations remain
byte-identical to the original `044e518c` source:

| Selected callback                                                 | Original lines | Current lines | Complete callback SHA256                                           |
| ----------------------------------------------------------------- | -------------- | ------------- | ------------------------------------------------------------------ |
| documentColor reports every supported CSS colour and nothing else | 108–139        | 109–140       | `8c3f0059217daa278bfeada049181b521a4065cbe739822db31910e9d7117017` |
| colorPresentation offers hex, rgb, and hsl notation               | 141–157        | 142–158       | `2d73e00582c17fdeba065b3994c6935dac1cc29425f2662f179c2ee1330bf076` |

The original whole-file hash remains `b46e2bad…`; the current whole-file hash is
`c9e909d1…`. Seven whole-file original/current pairs are now byte-identical; the
catalog still has eight response candidates and two safety controls. The dense
case at current lines 160–189 receives no association with this historical guard
fix. Exact current-file hashes, callback hashes and selected boundary checks
reject changes to the helper, expectations, added dense case or unrelated bytes.
No configuration-source inverse, color runtime or wire expectation changes are
used to make this successor match. Complete-response, dependency-closure,
whole-fix and native-equivalence credit remain unchanged and unproved.
