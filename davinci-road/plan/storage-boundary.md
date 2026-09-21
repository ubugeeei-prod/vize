# Davinci storage boundary

S0 (`vize_s0`, retained package id `vize_carton`) is the storage vocabulary for
Davinci stage code. This keeps representation decisions visible at one layer
instead of letting each S1/S2/S3 consumer select a different standard-library
type.

| Need                     | Type                              | Rule                                                                                         |
| ------------------------ | --------------------------------- | -------------------------------------------------------------------------------------------- |
| owned text               | `vize_s0::String`                 | This is `CompactString`; do not name `std::string::String` in stage libraries.               |
| arena-owned sequence     | `vize_s0::Vec`                    | Use for Drop-free IR data allocated with S0.                                                 |
| small scratch sequence   | `vize_s0::SmallVec`               | Use when a measured or grammatical inline bound exists; test both inline and spill behavior. |
| unbounded owned sequence | `alloc::vec::Vec`                 | Retain only in the exact reviewed inventory below; new or removed sites update the ledger.   |
| hash collection          | `vize_s0::{FxHashMap, FxHashSet}` | Do not name a `std::collections` hash type in stage libraries.                               |

The `davinci-opt` files under `crates/vize_davinci/src/bin/davinci-opt/` are an
explicit host edge. They may use `std` for paths, environment, filesystem, I/O,
and exit codes. That exception does not extend to `vize_davinci` library code
or to S1, S2, S3, and S1-to-S2 libraries. Importing or aliasing the `std`, `vec`,
or `collections` modules does not bypass the boundary.

## Retained `alloc::vec::Vec` inventory

The reviewed per-file ledger is [`storage-inventory.tsv`](./storage-inventory.tsv).
"Direct" counts imports and fully-qualified paths; "bound" counts every type,
constructor, and method path reached through a direct `Vec` import or alias.
The executable ledger requires strict equality, so both growth and reduction
must update the file row in the same change. The aggregates (retained
`alloc::vec::Vec` totals, per category and per scope) are derived from those
rows into the generated [storage summary](./storage-summary.md) by
`rust-script tools/commands/davinci/storage-summary.rs --write`, never copied
by hand; the regenerated page shows the aggregate movement of every change.

| Category | Reason                                                                                                                    |
| -------- | ------------------------------------------------------------------------------------------------------------------------- |
| contract | Owned Folio, S2/S3 serialization data, and stage dumps have input-defined cardinality and form stable contracts.          |
| analysis | Diagnostics, side tables, fact tables, filters, and verifier results grow with the input; no inline bound is established. |
| lower    | Lowering worklists and owned results grow with source-tree shape. Bounded substructures may migrate independently.        |
| pass     | Facts, provenance, and traversal worklists grow with the number of operations.                                            |
| emit     | Ordered output buffers and collected emission inputs grow with the document.                                              |

This is not an endorsement of every retained allocation. A focused change may
replace a site with `SmallVec` after measuring a bound; that change lowers the
exact ledger row in the same commit (the regenerated summary lowers with it),
making reintroduction fail.
Mechanical conversion of source-sized buffers is not a goal because it can
move large payloads onto the stack or add spill bookkeeping without reducing
allocations.

The S2 provenance page (`folio/provenance.rs`) and the S2-to-S3 partition
page (`partition/folio.rs`) each retain one source-sized `Vec` of owned
records: the same contract storage as the other derived Folio pages, while
the live provenance records and the arena-owned `PartitionFacts` stay with
their stages.

S3 value payloads use an arena-owned operand sequence. The separate owned
`values_folio.rs` page retains a source-sized `Vec` for serialization and S0
strings for tuple fields; `verify/operands.rs` appends input-sized diagnostics
to the existing verifier buffer. These are contract and analysis storage,
respectively, not additional owned IR sequences.

The second `alloc::vec::Vec` import in `side_table.rs` is `#[cfg(test)]`
size/test evidence. The scanner excludes the complete attributed item or
module, so it cannot inflate production totals.

## Exact owned-storage inventory by scope

The per-scope counts for every owned-storage type are part of the generated
[storage summary](./storage-summary.md), derived from the per-file
[`storage-inventory.tsv`](./storage-inventory.tsv) ratchet. Zero rows matter:
in particular, any production `alloc::string::String` path creates a new file
or count and fails the gate instead of becoming a `no_std` escape from S0.

`tests/tooling/davinci-storage-policy.test.ts` masks comments, literals, and
`#[cfg(test)]` items; resolves root, self, group, module, and raw aliases; and
checks every production file, category, scope, and owned-storage type for exact
equality with the TSV; `tests/tooling/davinci-storage-summary.test.ts` checks
the generated summary for byte equality with the aggregates of the TSV rows.
