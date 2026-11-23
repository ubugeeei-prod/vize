# Davinci layer names

Davinci uses **L0–L4** for its numbered layers. The previous S0–S4 spellings
remain in historical implementation and measurement records, and in specific
versioned wire identities described below.

| Layer | Role                                                  | Workspace package |
| ----- | ----------------------------------------------------- | ----------------- |
| L0    | Source text, spans, arena and storage                 | `vize_l0`         |
| L1    | Lossless syntax, container segments and recovery      | `vize_l1`         |
| L2    | File-local bindings, references and component regions | `vize_l2`         |
| L3    | Scheduled operations and directed dependency edges    | `vize_l3`         |
| L4    | Target writers, emission documents and source maps    | `vize_l4`         |

The conversion packages are `vize_l1_to_l2` and `vize_l2_to_l3`. The compatibility
crate `vize_carton` depends on `vize_l0`. L4 has writers for DOM, SSR, Vapor and
virtual TypeScript. Each target has its own admission and parity requirements.
[Workspace dependencies](../../Cargo.toml) · [L4 targets](../../davinci/vize_l4/src/targets.rs)

## Rust views of an original Vue file

With `vize_l1_to_l2/ir-views`, `lower_sfc_native` returns an observation whose
`l1ir()` borrows the original syntax. A formatter or editor can read its ordered
segments and diagnostics even for a half-written tag. Biome's recoverable CST uses
missing and bogus syntax to keep incomplete input available to its consumers.
[Biome parser and CST](https://biomejs.dev/internals/architecture/#parser-and-cst) · [Davinci source and recovery test](../../davinci/vize_l2_to_l3/tests/ir_queries.rs)

When `admitted()` succeeds, `l2ir()` borrows the file's semantics. Its `source()`
returns the input buffer that its byte ranges address, and `root_region()` borrows
the region used for lowering. With `vize_l2_to_l3/graph-queries`, `lower_ir`
reads that region and `l3ir()` exposes the resulting operations and edges. A
single query can scan the edges. Repeated queries can share an `OpGraph` index.
[IR views](../../davinci/vize_l1_to_l2/src/native_file/ir_views.rs) · [Graph queries and their verification](./decisions/2026-11-22-ir-graph-queries.md)

## Rust package migration

The four S-named packages at `0.428.1` remain published under their original
names. The renamed packages first publish at `0.429.0`:

| Previous package | Current package |
| ---------------- | --------------- |
| `vize_s1`        | `vize_l1`       |
| `vize_s2`        | `vize_l2`       |
| `vize_s1_to_s2`  | `vize_l1_to_l2` |
| `vize_s2_to_s3`  | `vize_l2_to_l3` |

Update Cargo dependencies, Rust imports and stage-specific module/type names
when moving to these packages. The previous L3 package `vize_impeto` remains
published under its original name; the current workspace uses `vize_l3`. This
physical L3 rename preserves dump formats and product output, while its first
publication must use the exact validated release candidate.

The source API rename is intentional for the
`0.429` minor release; renaming a package does not redirect an old dependency.

The first-release registry semver gate uses each rename's exact published source
archive as its baseline: `0.428.1` for the four S-named packages above, and
`vize_impeto@0.429.0` for the physical L3 package rename. Repository comparisons use the
actual base Git tree. Only the selected Cargo package/library identity is mapped
to the current name; a base workspace keeps its old dependency key and path with
an explicit package identity link. Versions, dependency aliases and Rust source
API remain unchanged, so removed public APIs still participate in the comparison.
Later releases use the renamed package's published baseline normally.

## Compatible wire identities

L names describe the current implementation and user-facing layer rail. Existing
versioned contracts keep their identities:

- Artifact-key hash domains and printed `sN.v1` keys are unchanged. Physical
  layer metadata uses `lN`; it is separate from the stable wire identity.
- Folio keeps `[s1]`, `[disegno]` and the existing `s2-`/`s3-` page headers.
  Named derive overrides preserve those headers for L-named Rust types.
- Extension WIT fields, feature tokens such as `s2-page@1`, schema numbers and
  frozen contract/SDK fixtures retain their original bytes. The SDK exposes
  L-named constants and `pages::l1`/`pages::l2`, with S-named API aliases for
  existing guest source. Output request fields `s2`/`s3` stay wire-compatible.
- Resident query accounting keeps its existing `s1_block`/`s2_page` IDs.
- Existing profile, remark and Spolvero stage strings remain valid. The
  playground maps them to L names for display. CLI pipeline selectors accept
  L names and their existing S aliases; terminal summaries always display L.
  Saved Folio/profile IDs retain their versioned S wire identities, and remark
  selectors accept either spelling against those saved records.

Historical exact-head evidence continues to name the packages and stages that
were actually measured. This naming change does not mark a phase complete,
change a performance target or replace an acceptance record.
