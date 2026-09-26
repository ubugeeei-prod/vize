# Stage view paths and feed types

This is the second bounded naming slice of [#6832](https://github.com/ubugeeei-prod/vize/issues/6832),
stacked after physical L3 identity and the declaration dependency gate.

The move-only commit changes 130 paths without changing their bytes:
36 playground stage-view paths, six inspector/WASM/feed paths, and 88 DOM
L2 witness paths. `features/davinci/` becomes `features/stages/`, inspector
and WASM module `spolvero` becomes `stages`, and DOM witness prefix
`davinci_l2_` becomes `l2_`. Consumers and inventory path cells follow.

The typed feed family becomes `StageFeed`, `StagePage`, `StageRemark`,
`StageFeedSchemaMismatch`, `StageFeedRemark`, and `StageNegotiation`.
Its JSON schema, fields, serialization and negotiation behavior stay intact.
The deterministic script separates moves from references, rejects move
collisions or a dirty tracked worktree, and verifies an idempotent replay.
Source comparison reverses only these named references and module/test paths.

The existing `spolvero` and `spolveroProfile` payload members, schema version,
error messages, profiler keys, counter names, remark IDs, feature names, dump
headers and frozen fixtures stay byte-faithful. Negotiation function and schema
constant names remain for a subsequent naming slice; no compatibility alias
is added for the renamed types. Inspector execution and CLI behavior do not change.

The shared Folio/Dump graph, remaining file/type names, formal/Lean namespace,
serialized name migration, conversion registry and shared production CLI/view
generator are outstanding. #6832 remains open and #6833 does not start here.

Focused source and pure reader checks are local preparation. Rust workspace,
WASM, browser and differential execution require exact-head Actions after
the release owner lifts the publication hold.

## Cargo target registration correction

The first prepared path slice missed the explicit DOM filter test registration.
The target becomes `l2_filters`, retaining its `legacy` required feature.
A focused gate checks every workspace metadata target's source file and every
explicit manifest target's registration. Cargo metadata alone may omit malformed
explicit targets, so source-path existence without that cross-check is insufficient.
The original move/protected-byte proofs do not establish runnable manifests.
