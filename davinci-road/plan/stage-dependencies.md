# Davinci stage dependency policy

Stage names are the primary implementation vocabulary. Historical art names
remain Cargo package ids and release identities until a dedicated compatibility
change can rename them safely.

| Role                             | Preferred crate name | Retained package id |
| -------------------------------- | -------------------- | ------------------- |
| S0 source and storage foundation | `vize_s0`            | `vize_carton`       |
| S1 lossless surface tree         | `vize_s1`            | `vize_sinopia`      |
| S2 semantic IR                   | `vize_s2`            | `vize_disegno`      |
| S1 to S2 lowering                | `vize_s1_to_s2`      | `vize_ricalco`      |

`vize_davinci` is shared infrastructure rather than another artifact stage. It
owns ids, diagnostics, side tables, pass machinery, Folio contracts, and the
canonical alias metadata used by tools.

## One-way graph

The graph is ordered by build tier, not by semantic stage number. Dependencies
may point only to an earlier tier:

| Build tier | Preferred crate | Allowed Davinci dependencies                    |
| ---------- | --------------- | ----------------------------------------------- |
| 0          | `vize_s0`       | none                                            |
| 1          | `vize_davinci`  | `vize_s0`                                       |
| 1          | `vize_s1`       | `vize_s0`                                       |
| 2          | `vize_s2`       | `vize_s0`, `vize_davinci`                       |
| 3          | `vize_s1_to_s2` | `vize_s0`, `vize_davinci`, `vize_s1`, `vize_s2` |

S0 must never depend on a later tier. The conversion crate is the only current
crate that joins both artifact stages.

The [storage boundary](./storage-boundary.md) defines how every stage consumes
S0 strings and collections, inventories retained `alloc::vec::Vec` sites, and
keeps `std` confined to the explicit `davinci-opt` host edge.

Cargo manifests use dependency renames, so source imports stay on `vize_s0`,
`vize_s1`, `vize_s2`, and `vize_s1_to_s2` while package publication remains
compatible. `tests/tooling/davinci-stage-dependencies.test.ts` reads Cargo
metadata to pin every rename and reject a reversed tier edge or cycle.

## S0 host boundary

`vize_s0` names the accepted Carton foundation; it does not claim that the
entire Carton package is `no_std`. Carton still exposes host-side configuration,
path, LSP, and profiling modules for existing consumers. Davinci stage libraries
must import compact strings, small collections, source, span, and arena storage
through `vize_s0` without importing `std` storage directly. Splitting Carton's
core and host surfaces is a later compatibility change, not part of this alias
transition.
