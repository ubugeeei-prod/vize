# Canonical Dump paths and nominal types (#6832, partial slice)

This local slice follows the physical L3 package and stage path/type changes.
#6832 remains open; Stage 1 work is still blocked on its full completion.

## Scope and boundary

The first commit moves 45 authored module/test paths with no content changes.
A replayable script then changes nominal Rust APIs and their consumers:

- Shared `vize_davinci::dump::{Dump, Mode, Error}` and `value::DumpValue`.
- `dump::collector::{Collector, Page}` and `plan`, `croquis`, `repro` pages.
- L2/L3 `dump::{Page, Op, ...}` and concern-specific `dump::Page` namespaces.
- The `Dump` derive and `#[dump(name = ...)]` attribute.
- Neutral local aliases distinguish live IR rows from owned dump rows.

The old L2/L3 codename and reactivity type aliases are removed. Croquis/repro
pages remain in the existing substrate; this change does not relocate them.
The existing canonical-namespace test preserves the L2 Full wire roundtrip.

## Output and nominal labels

Existing explicit headers, parser errors, operations, counters, Cargo features,
JSON fields, fixture paths and extensions remain unchanged. The plan page gains
an explicit `fusion-plan-folio` header equal to its former derived default.

Renamed structs/tuple wrappers change their derived Debug labels. Public
`type_name` paths change too; the current fact-group call does not name a dump
page. These nominal changes are intentional API changes. Proc-macro API error
labels use the new derive/attribute names; runtime Display strings stay exact.

The 19 existing snapshot hits contain old names only in expression metadata,
with no old name in their bodies. The macro's new expression names are expected;
existing snapshot bytes/membership remain frozen. Insta's ordinary comparator
checks bodies; full metadata matching would also observe the renamed expression.

## Verification and remaining work

The script checks alias collisions and replays the immutable-parent token
transformation under the existing formatter. Pure fixtures exercise cross-level
pages, live/shadow rows, macro paths, attributes and protected literal handling.
Source caps, typed import targets, declaration order and protected bytes are
checked locally. Rust compilation, WASM and real compiler/linter byte equality
still require Actions evidence at the eventual published head.

Lean namespaces/formal paths, serialized/runtime naming, remaining test names,
CLI option removal and the shared production observer are later slices.

## Scope-resolution correction

A follow-up preserves the first prepared head and fixes inline test-module and
separate binary-crate import resolution. Root-only malformed `Page` imports and
binary `crate::dump` imports must be rejected by source validation. Regression
fixtures cover both cases. The 33 reviewed storage inventory paths follow the
move manifest; all category/count fields and row order remain unchanged.

## Replay after CI and foundation repairs

The prepared parent is foundation stage head `f888b9ec4`, followed by the
registry correction `37197d6f6`. The 45-path move remains a separate R100
commit. Registry, move, Rust-scope correction and historical replay-pin patches
retain their original stable patch ids. The API patch differs only in the
central record link, which retains both registry and dump decisions. The DOM
target correction retains the parent's explicit `tests/l2_filters.rs` path,
renames its private test target to `l2_filters`, and keeps the `legacy` feature.

The existing replay driver now accepts `--base` for the clean pre-move source
revision. Verify this prepared tree with `--base 37197d6f6 --verify`; it checks
184 rewritten Rust files and the storage path inventory without writes. This
retains current main's public SSR compatibility changes rather than comparing
or overwriting them from the old immutable parent. The default historical
parent remains available for reproducing earlier proofs.

Existing Node contracts passed 69 tests, including the wrapper's ten Python
namespace/literal boundary cases and actual locked/offline Cargo target
registration. Croquis consumption, consumer migration, rule parity and storage
summary checks all pass without regeneration. Production Rust compilation,
real CLI execution, WASM/browser behavior and new exact-head Actions remain
unverified; prior captured outputs do not certify this new composition.

The remaining named paths, serialized concern headers, versioned counter
producer migration and the real shared CLI/playground production generator
are still open. The roundtrip-only CLI candidate does not replace the legacy
optimizer. The existing substrate dissolution remains #6833's separate scope;
this naming slice does not claim it has completed.

The current stability table names the actual `Dump` derive/trait and L2/L3
`dump::Page` entrypoints after the alias removals. Product tiers and promises
are unchanged; historical API names are not presented as current exports.
