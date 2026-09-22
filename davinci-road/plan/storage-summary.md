<!-- GENERATED FILE - do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/storage-summary.rs --write
     Verify:     rust-script tools/commands/davinci/storage-summary.rs --check
     Source:     davinci-road/plan/storage-inventory.tsv (the reviewed per-file ratchet) -->

# Davinci storage summary

Aggregates of the per-file [`storage-inventory.tsv`](./storage-inventory.tsv)
ledger behind the [storage boundary](./storage-boundary.md). Every number here
is derived from the ledger rows, so a change that moves a count updates its
file row and regenerates this page; the storage policy test holds the rows to
strict equality with the sources and this page to byte equality with the rows.

## Retained `alloc::vec::Vec`

The library trees in the reviewed inventory contain 118 production files,
130 direct `alloc::vec::Vec` paths, and 483 bound `Vec`/`StdVec` uses.

| Category | Files | Direct paths | Bound uses |
| -------- | ----: | -----------: | ---------: |
| contract |    26 |           37 |        103 |
| analysis |    25 |           26 |        101 |
| lower    |    16 |           16 |         71 |
| pass     |    17 |           17 |         72 |
| emit     |    34 |           34 |        136 |

## Owned storage by scope

| Scope    | Type                    | Files | Direct paths | Bound uses |
| -------- | ----------------------- | ----: | -----------: | ---------: |
| infra    | `alloc::vec::Vec`       |    28 |           28 |        120 |
| infra    | `alloc::string::String` |     0 |            0 |          0 |
| infra    | `vize_s0::String`       |    35 |           35 |        217 |
| infra    | `vize_s0::Vec`          |     0 |            0 |          0 |
| infra    | `vize_s0::SmallVec`     |     0 |            0 |          0 |
| s1       | `alloc::vec::Vec`       |     0 |            0 |          0 |
| s1       | `alloc::string::String` |     0 |            0 |          0 |
| s1       | `vize_s0::String`       |     0 |            0 |          0 |
| s1       | `vize_s0::Vec`          |    14 |           14 |         74 |
| s1       | `vize_s0::SmallVec`     |     0 |            0 |          0 |
| s2       | `alloc::vec::Vec`       |    11 |           23 |         45 |
| s2       | `alloc::string::String` |     0 |            0 |          0 |
| s2       | `vize_s0::String`       |    13 |           13 |         59 |
| s2       | `vize_s0::Vec`          |     9 |            9 |         17 |
| s2       | `vize_s0::SmallVec`     |     0 |            0 |          0 |
| s3       | `alloc::vec::Vec`       |    13 |           13 |         57 |
| s3       | `alloc::string::String` |     0 |            0 |          0 |
| s3       | `vize_s0::String`       |     6 |            6 |         27 |
| s3       | `vize_s0::Vec`          |     2 |            2 |         14 |
| s3       | `vize_s0::SmallVec`     |     0 |            0 |          0 |
| s1_to_s2 | `alloc::vec::Vec`       |    65 |           65 |        260 |
| s1_to_s2 | `alloc::string::String` |     0 |            0 |          0 |
| s1_to_s2 | `vize_s0::String`       |    95 |           99 |        477 |
| s1_to_s2 | `vize_s0::Vec`          |    16 |           18 |         69 |
| s1_to_s2 | `vize_s0::SmallVec`     |     5 |            5 |         10 |
| s2_to_s3 | `alloc::vec::Vec`       |     1 |            1 |          1 |
| s2_to_s3 | `alloc::string::String` |     0 |            0 |          0 |
| s2_to_s3 | `vize_s0::String`       |     0 |            0 |          0 |
| s2_to_s3 | `vize_s0::Vec`          |     1 |            1 |          2 |
| s2_to_s3 | `vize_s0::SmallVec`     |     0 |            0 |          0 |
