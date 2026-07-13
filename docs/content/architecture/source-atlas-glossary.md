---
title: Atlas artifact graph glossary
---

# Atlas artifact graph glossary

This glossary complements [Atlas artifact graph](/architecture/source-atlas).

## Core terms

| Term                           | Definition                                                                                                                 |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------------------- |
| Compilation                    | One source store, provider registry, typed-input store, artifact cache, counters, and trace history.                       |
| Source                         | Stable `SourceId`, name, text, revision, and root/embedded provenance.                                                     |
| Product                        | An open typed identity and cached value contract owned outside Atlas.                                                      |
| Request                        | A complete `(SourceId, ProductId)` identity; roots and dependencies may span several sources.                              |
| Provider                       | An independently registered implementation that declares applicability, inputs, and product dependencies before execution. |
| Plan                           | Immutable requested dependency closure with selected provider identities and relevant input revisions.                     |
| Compilation snapshot           | Immutable clone of sources, providers, inputs, and cache that can be forked without observing later mutations.             |
| Recipe                         | A consumer's set of requested root products; it contains no manual parse/analyze/lower orchestration.                      |
| Outcome                        | Typed products, execution/cache statuses, and trace from one plan execution.                                               |
| Provider observation           | Cached diagnostic, fallback, or note attributed to its request, concrete provider, source, and optional range.             |
| Relevant input                 | An open typed configuration dimension declared by a provider and propagated to transitive consumers.                       |
| Persistent artifact allocation | A product value retained in `ArtifactCache`; unrequested products create none.                                             |

The rejected terms `SourceAtlasPlate`, `SourceAtlasRoute`, and
`SourceAtlasRegistry` are not compatibility APIs. The graph's real plan and
trace are the source of observability.

## Representation terms

| Term       | Definition                                                                                                                                                 |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Module     | Owned JS/TS imports, exports, declarations, references, diagnostics, and OXC CFG facts; independent of Croquis and Relief.                                 |
| Relief     | Owned/source-faithful Vue-template syntax: authored shape and locations.                                                                                   |
| Croquis    | Owned/frontend-neutral semantic facts: identity, scopes, bindings, usage, and reactivity.                                                                  |
| Flow       | Frontend-neutral single-file control/data/effect graph with graph analyses.                                                                                |
| Croquis CF | Separately requested cross-file module/component aggregation ownership: a lightweight project index and a distinct full analyzer product; neither is Flow. |
| Rendu      | Owned/frontend-neutral structured render HIR consumed by DOM, SSR, Vapor, and custom backends.                                                             |
| Virtual TS | Canon's mapped typecheck projection; a consumer product, not the canonical IR.                                                                             |
| DOM output | JavaScript module and mappings emitted from Rendu.                                                                                                         |
| SSR output | Server-render module and mappings emitted from Rendu.                                                                                                      |
| Vapor plan | Owned target plan lowered from Rendu; target specialization remains outside Rendu.                                                                         |

## Boundary tests

Use these questions when placing a new type:

- Is the fact authored syntax and location? It belongs to Relief or the owning
  frontend syntax product.
- Is it a JavaScript/TypeScript module fact or OXC CFG projection? It belongs to
  Module.
- Is the fact derived identity, scope, binding, usage, or reactivity? It belongs
  to Croquis.
- Is it a block/edge/value/effect relation used for analysis? It belongs to
  Flow.
- Is it structured render intent shared by targets? It belongs to Rendu.
- Is it target-specific emission or planning? It belongs to the target Atelier.
- Is it source identity, planning, caching, invalidation, or tracing? It belongs
  to Atlas.

Adding any of these must not require adding a domain enum variant to Atlas.

## Cost vocabulary

| Observation            | Meaning                                                                                                     |
| ---------------------- | ----------------------------------------------------------------------------------------------------------- |
| Query                  | A root request or a provider's typed dependency read.                                                       |
| Execution              | A provider invocation whose value was not already cached.                                                   |
| Cache hit              | Reuse of the selected provider's product for the same source revision and relevant inputs.                  |
| Unselected provider    | Registered candidate whose `supports` result was false; its dependency hook and execution are both skipped. |
| Unrequested product    | Product outside the planned transitive closure; it has zero execution and no cache entry.                   |
| Selective invalidation | Eviction of products affected by one source tree or relevant typed input, preserving unrelated entries.     |

## References

- [Atlas artifact graph](/architecture/source-atlas)
- [Artifact graph cost baseline](/architecture/artifact-graph-cost-baseline)
- [#1634](https://github.com/ubugeeei-prod/vize/issues/1634)
