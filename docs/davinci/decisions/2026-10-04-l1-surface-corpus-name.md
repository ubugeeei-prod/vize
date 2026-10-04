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
