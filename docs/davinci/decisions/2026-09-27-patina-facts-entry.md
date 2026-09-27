# Private Patina S2 fact interning preparation

This is a private candidate for root review. The integration owner records the
paired issue and central decision before composition or publication.

## Concrete work reduction

`Facts::intern` previously called `ids.get` and, for a new `(namespace, local)`
key, separately called `ids.insert`. It now uses the existing map's `entry` API,
so the insertion uses the entry's cached hash instead of hashing the same key
again. Occupied entries return their original ID. Vacant entries check the
256-element limit before inserting a key or updating names/cased names.

The existing `vize_l0::FxHashMap` is a re-export of rustc-hash 2.1.3's standard
HashMap alias. Its Entry type is `std::collections::hash_map::Entry`. The
standard map is backed by hashbrown; the candidate keeps its hasher, map type,
dependencies, table data, traversal, exact intern keys and insertion order.
Case-insensitive matching remains exclusively in `Facts::id`.

This addresses a concrete duplicate hash operation, rather than asserting that
the measured regression came from changed Facts source. The instruction agent's
final 56fc attribution records Patina S2 at **543205 instructions**, above its
unchanged **539107** cap. Its 5e→56fc function attribution was:

| Function      | 5e self instructions / calls | 56fc self instructions / calls |
| ------------- | ---------------------------: | -----------------------------: |
| Facts::intern |                  52132 / 494 |                      1528 / 52 |
| Facts::parse  |                   121832 / 1 |                     159737 / 1 |
| Facts::id     |                  50556 / 438 |                    28506 / 438 |

These are profile observations, not an algorithm regression attribution or a
prediction of savings. The visible 52 intern calls do not count every inlined
operation. Source at base `56fcfd91a048c7e02693f12fb49f2f920b99920a` still performs
get-then-insert on its new-key path.
The source artifact used for attribution is
`/tmp/vize-native6962-final-56fc-exclusive-attribution.json`, target key
`patina_s2_markup_one_root`; raw measurements and caps remain untouched.

## Allocation boundary

Installed Rust 1.98's std HashMap Entry calls hashbrown 0.17.1's `rustc_entry`.
That implementation calls `reserve(1)` when the key is vacant, before returning
the vacant entry. Consequently reserve can run before this function rejects an
overflowing name. The source change guarantees no logical key, name, or cased
name insertion on overflow. It does **not** establish unchanged allocations or
memory usage. Native allocation gates and all measured targets remain decisive.

## Authored whole-state oracles

Five tests in the loader's new cfg(test) child module compare complete logical
state: ordered names, every name→ID mapping, ordered cased names, all row bits,
conditions/text/anchors, child rows, and the fallback empty row.

- All 146 committed-table occupied hits preserve the complete state.
- First insertions into an empty universe preserve namespace identity and SVG
  mixed-case order.
- Insertions into a nonempty table preserve explicit IDs 146–151, distinct
  exact-case keys, HTML/SVG/MathML case fallback, and namespace isolation.
- Exactly 256 names remain addressable. Every occupied hit still succeeds at
  capacity; repeated fresh names in all namespaces return None and leave the
  complete logical state unchanged.
- A whole appended loader fixture fills IDs 146–255 and rejects three members
  plus a new parent. Its exact four defect records, complete surviving child
  bitset, and all other state are fixed by the oracle.

The indexed boundary fixture lives only in test code. Existing fact tables,
raw benchmarks, measurement inputs, drivers, ceilings, workflows, and captured
outputs are unchanged. No production code or declarations are relocated.

Metadata confirms the committed table has 146 unique identities and no existing
div child row, and the authored IDs 146–255 produce bit words
`[0, 0, 18446744073709289472, 18446744073709551615]`. This is fixture authorship
evidence only, not execution of the Rust tests.

## Review and unfinished work

Local preparation is limited to source inspection, fixture metadata, Rust
formatting, source-size and diff checks, which passed. The loader has 174 lines
and the test module has 223 lines; both are below the 350-line cap. Source proof
confirms that all production bytes outside intern, its Entry import, and the
cfg(test) module declaration are unchanged. No Cargo command, Rust test, native
benchmark, workflow dispatch, PR, or public comment is run for this candidate.

TODO after root source review: run the new loader tests and the existing Patina
fact-table/checker tests in Actions, then run the unchanged exact-head native
gate for all 100 targets and allocations. The target's instruction cap must
pass without raising any ceiling. Savings, runtime correctness, and allocation
identity remain unverified until those results exist.
