# Shared resolved type world dependencies (2026-11-02)

Tracking: [#11370](https://github.com/ubugeeei-prod/vize/issues/11370).
Amends the first slice of
[typechecker source snapshot performance](./2026-10-04-typechecker-snapshot-performance.md)
and the dependency-facts clause of
[batch typechecker source snapshots](./2026-10-04-typechecker-batch-source-snapshot.md).

## Decision

The first slice cached unresolved dependency module facts per
`TypeSourceSnapshot` and cloned them before resolving module targets in each
world. Every component still paid for its whole reachable module graph: a deep
clone of each `TypeModule` with its declaration text, plus one target
resolution per import, export and star export. `canon.croquis.augment_type_props`
therefore scaled with components × reachable modules, and one prop type imported
through a barrel loaded the barrel's cost into every importing component.

Dependency facts are now resolved once per snapshot. `TypeSourceSnapshot` keeps
one `ResolvedDependency` per dependency path — an `Arc<TypeModule>` with every
target filled plus the modules the walk follows from it — behind the same
single-flight `Arc<OnceLock<..>>` shape as the other snapshot cells, and
`ResolvedTypeWorld::modules` holds `Arc<TypeModule>` values. The separate
unresolved parse cache is removed; the resolved cell parses the dependency
itself. Resolution depends only on the dependency's own path, its text and the
snapshot, never on the root component, so sharing cannot change a lookup. The
root module keeps its private, uncached parse (unsaved buffers, combined
normal/setup scripts), the walk pushes the same modules in the same order,
`MAX_MODULES` is checked at the same point, failures still insert
`TypeModule::default()`, and `resolve`, `resolve_import`, `declaration`,
`unknown_contract` and `resolve_properties` are unchanged.

## Scope and tradeoff

`vize_croquis::types::world::ResolvedTypeWorld::modules` changes its value type,
so `Clone` of a world is shallow for dependencies and a hand-built world wraps
its modules in `Arc::new` (or edits them through `Arc::make_mut`). A snapshot
retains one resolved copy per reachable dependency instead of one clone per
world, which is less memory as soon as two worlds share a dependency. Snapshot
lifetime is unchanged: one per editor revision in Maestro, one per
`register_paths` batch or reachable walk in Canon, so no new staleness window
appears. The first touch of a dependency closure still parses and resolves it
once per snapshot; that eager closure, not the per-world rebuild, is what
remains of the cost recorded in #11370.

## Evidence

[Paired run 41382042484](https://github.com/ubugeeei-prod/vize/actions/runs/41382042484)
(`blacksmith-32vcpu-ubuntu-2404`, base `07d1feb1bb`, head `7fd84eef83`, nine
alternating pairs) reports identical module-fact signatures for all twelve
workloads. Warmed 128-host medians:

| Workload     | Threads |     Base |    Head | Speedup |
| ------------ | ------: | -------: | ------: | ------: |
| Fanout       |       1 | 32.53 ms | 3.24 ms |  10.05x |
| Deep barrels |       1 | 29.97 ms | 1.52 ms |  19.75x |
| Vue/TSX      |       1 | 30.74 ms | 3.25 ms |   9.46x |
| Fanout       |       4 | 18.56 ms | 1.29 ms |  14.37x |
| Deep barrels |       4 | 19.88 ms | 0.92 ms |  21.62x |
| Vue/TSX      |       4 | 18.24 ms | 1.23 ms |  14.82x |

Cold one-host cases moved from 1.63 / 1.50 / 2.13 ms to 1.43 / 1.37 / 1.91 ms
and local-only cases stayed within 0.88–1.03x. The same harness on an Apple M1
(`ci-opt`) gives 2.1–2.9x warm and 1.00–1.07x cold. The run's `paired-cli` job
stopped before building any CLI because `canon-alias-collector.test.mjs` fails
on `main`; CLI parity comes from a 61-package workspace sweep whose diagnostics
are byte-identical before and after, with warm wall time unchanged at 112 s.
In that CLI the per-component world cost halves (p50 8.2 → 4.1 ms for twenty
components importing one type through a 400-module barrel) while the first
touch of a barrel still parses and resolves the whole closure once per process,
and parallel workers wait on that same first parse. The unit tests
`dependency_modules_are_resolved_once_per_snapshot_and_shared_between_worlds`
and `barrel_dependencies_are_resolved_once_however_many_worlds_import_them`
pin the sharing (`Arc::ptr_eq`), the cache contents and fact equality across
worlds and snapshots.
