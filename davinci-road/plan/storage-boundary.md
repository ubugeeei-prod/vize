# Davinci storage boundary

S0 (`vize_s0`, retained package id `vize_carton`) is the storage vocabulary for
Davinci stage code. This keeps representation decisions visible at one layer
instead of letting each S1/S2 consumer select a different standard-library
type.

| Need                     | Type                              | Rule                                                                                         |
| ------------------------ | --------------------------------- | -------------------------------------------------------------------------------------------- |
| owned text               | `vize_s0::String`                 | This is `CompactString`; do not name `std::string::String` in stage libraries.               |
| arena-owned sequence     | `vize_s0::Vec`                    | Use for Drop-free IR data allocated with S0.                                                 |
| small scratch sequence   | `vize_s0::SmallVec`               | Use when a measured or grammatical inline bound exists; test both inline and spill behavior. |
| unbounded owned sequence | `alloc::vec::Vec`                 | Retain only in the reviewed inventory below; new sites require a category and evidence.      |
| hash collection          | `vize_s0::{FxHashMap, FxHashSet}` | Do not name a `std::collections` hash type in stage libraries.                               |

The `davinci-opt` files under `crates/vize_davinci/src/bin/davinci-opt/` are an
explicit host edge. They may use `std` for paths, environment, filesystem, I/O,
and exit codes. That exception does not extend to `vize_davinci` library code
or to S1, S2, and S1-to-S2 libraries.

## Retained `alloc::vec::Vec` inventory

At policy introduction the four library trees contain 55 reviewed files and
72 direct `alloc::vec::Vec` paths. The executable inventory is file-specific,
caps each file at its observed count, and always permits the count to shrink.
The retained categories are:

| Category | Files / paths | Reason                                                                                                             |
| -------- | ------------: | ------------------------------------------------------------------------------------------------------------------ |
| contract |       13 / 24 | Owned Folio and S2 serialization data has input-defined cardinality and forms a stable contract.                   |
| analysis |         7 / 9 | Diagnostics, side tables, filters, and verifier results grow with the input; no inline bound is established.       |
| lower    |       12 / 16 | Lowering worklists and owned results grow with source-tree shape. Bounded substructures may migrate independently. |
| pass     |       11 / 11 | Facts, provenance, and traversal worklists grow with the number of operations.                                     |
| emit     |       12 / 12 | Ordered output buffers and collected emission inputs grow with the document.                                       |

This is not an endorsement of every retained allocation. A focused change may
replace any site with `SmallVec` after measuring a bound; the ratchet needs no
allowlist update when a use disappears. Mechanical conversion of source-sized
buffers is not a goal because it can move large payloads onto the stack or add
spill bookkeeping without reducing allocations.

`tests/tooling/davinci-storage-policy.test.ts` enforces the direct-`std` ban,
the exact host boundary, category reasons, and non-growing per-file maxima.
