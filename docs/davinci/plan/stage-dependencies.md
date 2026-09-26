# Davinci stage dependency policy

Stage names are the primary implementation vocabulary. Historical art names
remain Cargo package ids and release identities only where a dedicated
compatibility change is still required.

| Role                             | Preferred crate name | Current package id |
| -------------------------------- | -------------------- | ------------------ |
| L0 source and storage foundation | `vize_l0`            | `vize_carton`      |
| L1 lossless surface tree         | `vize_l1`            | `vize_l1`          |
| L2 semantic IR                   | `vize_l2`            | `vize_l2`          |
| L3 reactivity/backend scheduling | `vize_l3`            | `vize_impeto`      |
| L1 to L2 lowering                | `vize_l1_to_l2`      | `vize_l1_to_l2`    |
| L2 to L3 lowering                | `vize_l2_to_l3`      | `vize_l2_to_l3`    |

`vize_davinci` is shared infrastructure rather than another artifact stage. It
owns ids, diagnostics, side tables, pass machinery, Folio contracts, and the
canonical alias metadata used by tools.

## One-way graph

The graph is ordered by build tier, not by semantic stage number. Dependencies
may point only to an earlier tier:

| Build tier | Preferred crate | Allowed Davinci dependencies                    |
| ---------- | --------------- | ----------------------------------------------- |
| 0          | `vize_l0`       | none                                            |
| 1          | `vize_davinci`  | `vize_l0`                                       |
| 1          | `vize_l1`       | `vize_l0`                                       |
| 2          | `vize_l2`       | `vize_l0`, `vize_davinci`                       |
| 3          | `vize_l3`       | `vize_l0`, `vize_davinci`                       |
| 3          | `vize_l1_to_l2` | `vize_l0`, `vize_davinci`, `vize_l1`, `vize_l2` |
| 4          | `vize_l2_to_l3` | `vize_l0`, `vize_davinci`, `vize_l2`, `vize_l3` |

L0 must never depend on a later tier. Conversion crates are the only current
crates that join artifact stages.

The resident tier (`vize_resident`, P5-4a) is a consumer above every stage
tier: it runs the stage functions as salsa queries for long-lived processes
and is the only crate allowed to depend on `salsa`
(`tests/tooling/davinci-resident-salsa.test.ts`). No stage crate may depend
on it, and the one-shot CLI never links it.

The [storage boundary](./storage-boundary.md) defines how every stage consumes
L0 strings and collections, inventories retained `alloc::vec::Vec` sites, and
keeps `std` confined to the explicit `davinci-opt` host edge.

Cargo manifests use dependency renames where the package id still differs from
the preferred crate name. Today those exceptions are L0/Carton and L3/Impeto,
so source imports stay on `vize_l0`, `vize_l1`, `vize_l2`, `vize_l3`, and
`vize_l1_to_l2` while package publication remains compatible.
`tests/tooling/davinci-stage-dependencies.test.ts` reads Cargo metadata to pin
every spelling and reject a reversed tier edge or cycle.

## L0 host boundary

`vize_l0` names the accepted Carton foundation; it does not claim that the
entire Carton package is `no_std`. Carton still exposes host-side configuration,
path, LSP, and profiling modules for existing consumers. Davinci stage libraries
must import compact strings, small collections, source, span, and arena storage
through `vize_l0` without importing `std` storage directly. Splitting Carton's
core and host surfaces is a later compatibility change, not part of this alias
transition.
