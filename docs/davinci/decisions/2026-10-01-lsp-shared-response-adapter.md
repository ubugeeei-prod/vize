# Source-bound LSP response adapter — #6883 and #6891

## Scope and requirements

The narrow historical census is pinned to `cc87bb5960ea9e49e82672205df919de58bb4b24`:
255 nonmerge subjects beginning `fix` among 447 nonmerge Maestro-touch commits,
or 451 commits including four merges. The original 238/421 and the earlier
239/424 ledger describe different historical snapshots. None is a semantic
coverage percentage. Every selected SHA remains in the review ledger, including
scoped bookkeeping changes whose shared-provider effects still need review.
The [source requirement ledger](../plan/lsp-fix-history-requirements.tsv) records
207 response, 43 control and five scoped bookkeeping entries. Its historical
and current test paths are candidate lookup aids, not proved equivalent inputs
or passing complete responses. Requirements outside this narrow path selection,
the other non-fix touching commits and shared provider history still need review.

The shared `vize.differential.manifest` and `vize.differential.result` v1 contract
now has a prepared LSP adapter. It consumes the existing source-build receipt
and preserves one result for each authored case and `stdio` target. It does
not introduce another product result schema or alter the LSP implementation.

Seven isolated sessions plan ten complete selected JSON-RPC response envelopes:

| Fix                                        | Original inputs and complete contracts                                                                                                                                                                |
| ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `447716ebc8a2d52e810d2be1de6a8166a95e4c32` | Existing inactive-import input/dependencies and all three ordered document links.                                                                                                                     |
| `4d04bc3e53acfddf835f2db70e613abe85ce8df4` | Existing CRLF source and four complete on-type edit/no-op/null responses.                                                                                                                             |
| `89771a14f17fb535201f261ca154aa35c6f77d7e` | Exact original FourDivs, MultiBlock, comment, one-line and empty-block inputs. Complete folding objects retain optional-field omission and array order; the two empty-region controls require `null`. |

The folding references are authored contracts, rather than output from the new
adapter. MultiBlock preserves the original four block regions and includes the
two style-rule regions required by the current pinned public implementation.
The original test projected folds into tuples; this adapter compares the entire
serialized response, including any unexpected character fields or properties.
The earlier two fixtures' dated source proofs remain separate historical evidence.

## Runtime and retained evidence

The adapter requires the exact checkout's `target/ci/vize`, its SHA256, version
probe and matching `vize.differential.build` receipt before launching `vize lsp`.
An absent or stale executable fails every planned case without trying debug,
release, PATH or Cargo fallback. Inputs, metadata and authored references have
pinned hashes, and safe runtime paths cannot escape the isolated workspace.

Each session records every framed byte sent and received, stream digests, stderr,
exit status and signal. The record includes initialization, didOpen, exact entry
URI/version-1 diagnostics readiness, selected requests and graceful shutdown.
Its validator reconstructs complete frames with strict UTF-8, accounts for every
response ID in order, rejects duplicate/truncated/unplanned responses, and
recomputes the complete JSON comparison. Only the workspace URI token in the
reference is materialized; actual responses are never projected or sorted.

The dependent source-built Actions consumer writes `target/differential/lsp.json` even
when a comparison fails. The existing differential evidence upload recursively
retains product reports, formatter/linter raw directories and the CLI receipt.
Artifact names retain the formatter prefix and add workflow job/shard identity
to the run/attempt identity. Extracted artifacts contain `differential/` and
`ci/vize.differential-build.json`. PR and merge tooling retain the same evidence.
The consumer runs the provider through existing merge and full tooling jobs,
using the explicit T1 runtime inventory alongside the existing production LSP,
formatter and linter scenarios. Pure adapter laws remain in the quick PR tier.
It emits the report before asserting comparison success. The shared observer
provider already merged as #7362 supplies this recursive evidence upload; the
LSP consumer does not edit that action or either workflow caller. No job, build
stage, instruction ceiling or differential window is added or changed.

## Proof and remaining work

Local transport/validator laws exercise synthetic frames and an actual Node
transport subprocess. They are adapter laws, not production LSP observations.
The new source-built consumer must pass exact-head Actions and the protected
merge queue before these new registrations receive fresh runtime credit.

All native rows are explicitly `unsupported`, paired comparisons are
`not-compared`, and native handling/equivalence numerators are zero. Complete
initialization capability expectations, every diagnostic/state transition,
close/reopen, cancellation/churn, UTF-16 controls, checker non-start, transport
generation and every remaining historical response requirement are unfinished.
Three selected fixes and seven registered sessions do not close #6883, and the
legacy LSP route cannot be replaced on this evidence.
