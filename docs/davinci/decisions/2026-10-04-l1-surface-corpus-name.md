# L1 feature-gated surface corpus target naming

Decision for [#6832](https://github.com/ubugeeei-prod/vize/issues/6832),
2026-10-04. Audited source:
`7516c514bc58a73f1415585021ea34d801b5836d`.

## Bounded change

Move `davinci/vize_l1/tests/davinci_surface_corpus.rs` to `surface_corpus.rs`
in a byte-identical move-only commit. A separate integration commit changes
the explicit Cargo test target to `surface_corpus`, its current manifest and
source command comments, the shared fixture helper's file reference, and the
live P2-7 table link. The original P2-7 record's path and run command remain
historical evidence. No parser, provider, dependency or product dispatch changes.

The existing body checks all 16 well-formed and 26 malformed committed inputs,
round-trip bytes and exact typed-hole counts. Preserve every fixture value,
test assertion, serialized string and diagnostic, including the optional
`VIZE_DAVINCI_DIFFERENTIAL_CORPUS` widening contract. Preserve the
`legacy-differential` requirement and the deprecated published
`davinci-differential` alias under the
[published feature policy](./2026-09-29-published-feature-compatibility.md).
The legacy oracle remains a dev-only edge; no dependency-direction exception
or naming-gate allowance is added.

## Hosted execution

At the audited source, ordinary workspace test archives omit this required-feature
target. The full differential composite also has no L1 surface invocation.
Default Check success therefore cannot establish this target's execution.

Add this command to the PR Rust builder when its affected package plan includes
`vize_l1`, and to the existing full differential composite used by the merge
queue and manual Check:

```sh
cargo test -p vize_l1 --features legacy-differential --test surface_corpus -- --nocapture
```

The new required step executes the committed 42-case battery with the optional
corpus environment variable unset. Its retained stderr identifies this scope.
It does not hydrate the ecosystem corpus or claim ecosystem closure. Extend
the existing simulated-Cargo fail-fast law to reject the new target's failure
and retain every previous differential command and environment contract.
Fresh exact-head hosted execution and all protected queue checks, including
the unchanged 100 level plus four formatter ceilings, must succeed before
actual merge. Prior source or queue witnesses do not transfer to a new head.

## Replay and remaining work

Run `tools/support/levels/rename-l1-surface-corpus.ts moves` first, commit only
the move, then run `integrate` and commit references, CI and the paired decision.
`check` validates integration. Replay rejects conflicting or missing target
paths before moving, validates all integration anchors before any tracked
write, and is idempotent. On a conflict, replay only this slice on fresh main.

This slice follows the actually merged L0/L3 naming changes but shares no code
dependency; it is independently based on main without an artificial Stack.
Keep #6832 OPEN for remaining native/dialect paths, snapshots, published
feature compatibility, serialized naming, crate audit and production capture.
These corpus checks do not close product fix-history gates or establish a
native/default product replacement. Vue Fes completion remains separate.

## Preserved allocation failure and same-PR harness correction

Historical protected candidate `e1c0c8609e64035a9705cd76257e9d3fdaf966b4`
failed Check37181078166 shard2 with native text78 against unchanged ceiling75.
The exact original failure and all seven native/retained rows remain in
[#7764](https://github.com/ubugeeei-prod/vize/issues/7764). The diagnostic
#7770 is separate, and its64 serial attempts did not reproduce that failure.
The three excess calls' historical ownership remains UNKNOWN.

This same existing PR prepares a measurement-ownership correction using the
already-established standalone allocation-law protocol from L0/L1. Explicit
Cargo registration sets `davinci_vapor_native_budget` to `harness = false`;
the single unchanged case body then runs on process main instead of alongside
libtest's running-test map, timeout deque and result-channel bookkeeping.
This removes the known possible runner overlap, not an evidenced historical
explanation of78. It does not filter allocator calls, replace the allocator,
change process-global accounting or omit genuine workers spawned by a routine.

The original seven source literals, ceilings75/74/110/100/134/106/158, native/
retained order, one warm-up, compile/result/allocator drops, measured windows
and final aggregated failure assertion remain byte-identical. No production,
allocation counter, thread-local suppression, extra warm-up, retry or budget
change is introduced. Existing source/default contracts and every original
42-case L1 corpus boundary remain.

The realCASE/main CLI is the existing nextest discovery/exact-selector protocol:
listing prints one original case without measuring; ignored or excluded
selection returns without measuring; default/exact/substr selection executes
the complete original callback once. Unsupported arguments fail before entry
and an assertion remains a nonzero process exit. The focused hosted tooling law
compiles that actualCASE/main unchanged with an instrumented callback to reject
discovery execution, hidden selected-case omission and swallowed assertions.
That control is not an allocation measurement.

A narrowly scoped inherited nextest output override retains this one case's
seven success rows in PR/full JUnit artifacts. It changes no selection, retry,
concurrency, profile or other test-output policy. The pinned nextest0.9.146
archive/list must genuinely discover the original case and its assigned
unfiltered worker must execute it before runtime credit. Fresh source/protected
Actions must retain the whole raw vectors and all unchanged104×3 instruction
ceilings. Existing full feature-enabled L1 corpus execution stays mandatory.

The genuine integration preserves signed actual #7770 merge
`1744a7ee84c202ce78f03a0148f335a5022b72e3`, including its diagnostic labels
in the complete unchanged measurement body. That merge's full Nextest case
passed once; its prior successful stdout was not retained, so no seven-row
numeric vector is inferred from that receipt.

The optional historical `workflow_dispatch` diagnostic is separate from the
required automatic full Nextest gate. Its instrumented arm overlays current
budget source into frozen b9/E1 libtest manifests; a standalone `main` would
instead build a zero-case libtest target. The launcher therefore requires the
exact historical libtest budget bytes (SHA256
`c7dd7a9bbff3ddb3067ca13d6c5d4964103d49eb94798f18c069cded63e732dd`)
before tool queries, archive fetches, worktree creation or process execution.
The changed current source is explicitly refused. A pure guard law exercises
accepted authored bytes, a changed overlay and actual current-source refusal,
with every external-work boundary trapped; ordinary hosted tooling also runs
the retained diagnostic validator laws. Unsupported libtest thread arguments
are refused by the standalone CLI rather than accepted as a false contrast.
These controls grant no historical reconstruction or campaign execution credit.
TODO: independently review any recovery of the historical overlay/protocol;
the old diagnostic recipe is currently unusable on this standalone source,
and no trial or expired-archive retrieval is authorized.

Preparation has not compiled or executed the Rust controls locally, started a
campaign, pushed this source or queued #7757. Root source review and fresh complete Actions are
required. Oldfailed candidate acceptance is not rewritten; this correction
provides no historical-cause,10x/native/default product or full-history credit.
