# L0 dump law test target names

Decision for [#6832](https://github.com/ubugeeei-prod/vize/issues/6832), 2026-10-04.

The source audit is pinned to
`cf3b457baff6b6ab1735ffde825f5604455e0979`. L0 already owns the implementation
and integration laws after the [foundation law move](./2026-10-01-l0-runtime-laws.md).
Two auto-discovered Cargo test targets still carry the old page codename:

| Former target       | Current target     | Existing contract                                          |
| ------------------- | ------------------ | ---------------------------------------------------------- |
| `folio_derive_laws` | `dump_derive_laws` | Nine derived Dump normalization, round-trip and error laws |
| `folio_dump`        | `dump_hash_gate`   | Four Collector order and change-gate laws                  |

Move only these two `.rs` files under `davinci/vize_l0/tests/`, in a move-only
commit. Their Rust bytes, test functions, expected wire headers, diagnostics
and `.folio` output names remain exact. Cargo discovers the new targets
without manifest changes. This introduces no implementation, pipeline stage,
dependency, product default or published API change.

## Replay and current source qualification

Run `node tools/support/levels/rename-l0-dump-tests.ts moves`, commit only the
two moves, then run `integrate` and `check`. The move phase checks the entire
set for missing paths and collisions before changing the index. A second
execution performs no changes. Integration checks and buffers both reference
updates before writing.

The existing foundation ownership replay keeps its original `RELATIVE`
source names. Only its destination map and subsequent destination reads use
the current names. It can therefore replay the original substrate move
directly into today's L0 paths without rewriting the original source history.
Its `check` phase still validates the full ownership move inventory.

The current P2-4 documentation link points to `dump_derive_laws.rs`. Archived
P2-4/P2-13 records, phase-exit observations and original captured command
outputs retain their historical target names. Earlier test/build results are
evidence for those earlier sources; they do not establish discovery, build or
execution of the renamed targets. Fresh source-head Actions must compile and
execute the current targets. The protected queue must build and run the full
workspace archive and retain the instruction-count ceilings before merge.

## Remaining naming work

#6832 remains open. This bounded rename does not change retained `.folio`
protocols, collector page-name strings, the other native corpus/benchmark
paths, current pass-snapshot names, or extension fixture names. Their wire
and witness migrations need their own scoped audit. Legacy products and
published feature aliases keep the
[existing compatibility policy](./2026-09-29-published-feature-compatibility.md).
The published tokenizer consumer execution described there remains a separate
release gate. The declined Lean namespace rename remains superseded by the
maintainer's Kani decision; no formal fixture or recorded proof is renamed.
The separate playground legacy template compile and full crate-name audit
also remain outside this slice.
