<!-- GENERATED FILE - do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/storage-summary.rs --write
     Verify:     rust-script tools/commands/davinci/storage-summary.rs --check
     Source:     docs/davinci/plan/storage-inventory.tsv (the reviewed per-file ratchet) -->

# Davinci storage summary

Aggregates of the per-file [`storage-inventory.tsv`](./storage-inventory.tsv)
ledger behind the [storage boundary](./storage-boundary.md). Every number here
is derived from the ledger rows, so a change that moves a count updates its
file row and regenerates this page; the storage policy test holds the rows to
strict equality with the sources and this page to byte equality with the rows.

## Retained `alloc::vec::Vec`

The library trees in the reviewed inventory contain 121 production files,
133 direct `alloc::vec::Vec` paths, and 505 bound `Vec`/`StdVec` uses.

| Category | Files | Direct paths | Bound uses |
| -------- | ----: | -----------: | ---------: |
| contract |    27 |           38 |        114 |
| analysis |    25 |           26 |        101 |
| lower    |    16 |           16 |         72 |
| pass     |    19 |           19 |         82 |
| emit     |    34 |           34 |        136 |

## Owned storage by scope

| Scope    | Type                    | Files | Direct paths | Bound uses |
| -------- | ----------------------- | ----: | -----------: | ---------: |
| infra    | `alloc::vec::Vec`       |    29 |           29 |        131 |
| infra    | `alloc::string::String` |     0 |            0 |          0 |
| infra    | `vize_l0::String`       |    36 |           36 |        231 |
| infra    | `vize_l0::Vec`          |     0 |            0 |          0 |
| infra    | `vize_l0::SmallVec`     |     0 |            0 |          0 |
| l1       | `alloc::vec::Vec`       |     0 |            0 |          0 |
| l1       | `alloc::string::String` |     0 |            0 |          0 |
| l1       | `vize_l0::String`       |     0 |            0 |          0 |
| l1       | `vize_l0::Vec`          |    14 |           14 |         79 |
| l1       | `vize_l0::SmallVec`     |     0 |            0 |          0 |
| l2       | `alloc::vec::Vec`       |    11 |           23 |         45 |
| l2       | `alloc::string::String` |     0 |            0 |          0 |
| l2       | `vize_l0::String`       |    13 |           13 |         59 |
| l2       | `vize_l0::Vec`          |     9 |            9 |         17 |
| l2       | `vize_l0::SmallVec`     |     0 |            0 |          0 |
| l3       | `alloc::vec::Vec`       |    13 |           13 |         57 |
| l3       | `alloc::string::String` |     0 |            0 |          0 |
| l3       | `vize_l0::String`       |     6 |            6 |         27 |
| l3       | `vize_l0::Vec`          |     2 |            2 |         14 |
| l3       | `vize_l0::SmallVec`     |     0 |            0 |          0 |
| l1_to_l2 | `alloc::vec::Vec`       |    67 |           67 |        271 |
| l1_to_l2 | `alloc::string::String` |     0 |            0 |          0 |
| l1_to_l2 | `vize_l0::String`       |    97 |          105 |        482 |
| l1_to_l2 | `vize_l0::Vec`          |    18 |           20 |         69 |
| l1_to_l2 | `vize_l0::SmallVec`     |     5 |            5 |         12 |
| l2_to_l3 | `alloc::vec::Vec`       |     1 |            1 |          1 |
| l2_to_l3 | `alloc::string::String` |     0 |            0 |          0 |
| l2_to_l3 | `vize_l0::String`       |     0 |            0 |          0 |
| l2_to_l3 | `vize_l0::Vec`          |     1 |            1 |          2 |
| l2_to_l3 | `vize_l0::SmallVec`     |     0 |            0 |          0 |
