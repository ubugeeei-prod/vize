# Release integration PR body custody

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The immutable `v0.447.0` attempt failed its original integration catalog check:
[job 114666163513](https://github.com/ubugeeei-prod/vize/actions/runs/38204917577/job/114666163513).
Source PR #8739 pins `C=dd188515f8ec3c51662b4f4d69adcd39c86a0907` and
`H=ef34d1a0409fc711a6c5cefcf0708b0cce78f502`; integration PR #8740 pins
`M=6df0d4a6df129497ed8341d60cc114e61638097d`. The catalog refusal was
`Integration PR body/head/base changed since this event.`

The original complete job log, original H candidate verifier and post-failure
complete PR response are retained in
`tests/_fixtures/release/integration-body-447/`, with whole-byte SHA-256 and Git
blob identities. Source analysis isolates the full-body inequality clause:
the event repository/number/head were already checked and the base comparison
uses the same event field. The original event body was not retained. The later
body contains the generated policy and both bot decorations, but it does not
establish which editor changed it or the event order or time of that change.

## Decision

Move the unchanged generated integration policy into one shared producer.
Require its entire exact prefix and all four unique pin markers in both the
event and current body. Permit only one anchored Codesmith footer and one
anchored CodeRabbit summary after that prefix. Bind the Codesmith envelope to
the exact repository and integration PR, fixed asset URLs and complete query
grammar. Accept only bounded plain summary syntax for CodeRabbit. Unknown,
duplicated, incomplete or forged envelopes, altered policy and altered pins
remain refusals. A signature-shaped query value is syntax, not authenticated
provider identity; no signature, editor or timestamp claim is made.

All repository, PR identity, source, parent, tree, complete H catalog, version,
security and publication predicates stay authoritative. Only the old complete
event/current body equality is replaced by independent policy qualification.
The hosted check retains the whole event first and the whole current response
after its API read, before qualification; its always-running artifact step
preserves these raw snapshots even when authentication refuses. Local
`verify_candidate(None)` keeps its existing environment contract and makes no
durable snapshot claim. No additional release pipeline stage is introduced.

## Qualification and remaining work

Seven added Rust laws retain the genuine old failure and captured complete
body, exercise an explicitly authored generated-event model, and reject policy,
marker, head, repository, base and decoration mutations. They also require
complete event/current snapshot retention before refusal. The earlier 68 laws
retain their assertions; their valid event setup now includes the complete
generated policy required by the stricter body check.

The first local compile refused a private test import, then one authored
negative incorrectly rejected the explicitly permitted single ending newline.
Both failures are retained; the latter correction changes only that test model.
The final local source run passed 75/75 laws with no ignored tests. This is
source qualification, not a native build, hosted catalog or release success.
The complete source boundary has independent maintainer and Canon review.

The genuine replay onto actual main `5c54b1195fa8c0bf151e0a2862bf4238a3597b4e`
preserves the production boundary. Exact successor Actions, all protected
suites and actual merge remain required. The failed 447 source and metadata
commits are unchanged, with no retry or publication. A later immutable cut must
qualify every existing release gate and actual installed consumers; this repair
alone grants no publication or issue completion credit.
