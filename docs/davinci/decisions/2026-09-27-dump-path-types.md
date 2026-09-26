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
