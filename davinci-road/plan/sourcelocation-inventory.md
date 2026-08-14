<!-- GENERATED FILE — do not edit by hand.
     Regenerate: node tools/davinci/sourcelocation-inventory.mjs --write
     Verify:     node tools/davinci/sourcelocation-inventory.mjs --check
     Generator:  tools/davinci/sourcelocation-inventory.mjs -->

# `SourceLocation` consumer inventory

Every textual read of the relief `SourceLocation` members that
`vize_carton::Span` (`crates/vize_carton/src/span.rs`) deletes —
`source`, `start.line`, `start.column`, `end.line`, `end.column` —
across `crates/*/src`, plus the migration group each consumer moves to as
relief nodes switch to two-u32 byte spans instead of owned
`{ start: Position, end: Position, source: String }` triples
([architecture.md](../architecture.md), S0). This is the migration map Davinci
P1 executes (P0-9). P1-3 executed group 1: every `source` read is migrated,
the scan now counts zero, and regeneration fails if one comes back. The
line/column members remain until P1-4.

## Resolution method (and its limits)

- Comments and string literals are stripped first
  (`tools/davinci/lib/rust-source.mjs`); member paths are then counted
  **textually** on loc-shaped receivers only: chained `.loc.<member>`,
  `.loc().<member>`, `.location.<member>` accesses, and bare locals named
  `loc` / `location` / `*_loc` / `*_location`.
- Reads of `start.offset` / `end.offset` are **not** inventoried: they
  survive the migration verbatim as `span.start` / `span.end`
  (276 loc-shaped offset-read sites across
  8 crates at generation time).
- `#[cfg(test)]` code inside `src/` is included and reported in the
  "in test code" column: a site counts as test code when its file is a test
  module by name (`tests.rs`, `*_tests.rs`, `/tests/`) or sits at or
  after the file's first `#[cfg(test)]` attribute.
- The scan is not type-resolved. Known imprecision, spot-checked:
  - `vize_doctor` defines a namesake path+offset `SourceLocation`
    (`crates/vize_doctor/src/model/evidence.rs`) that is already
    span-shaped; its members (`path`, `start: u32`, `end: u32`) never
    form any of the five member paths, so it cannot contribute rows here and
    needs no migration.
  - `BlockLocation` (`crates/vize_atelier_sfc/src/types.rs`) also sits
    behind `loc` fields, but its `start`/`end` are plain `usize`
    offsets with flat `start_line`/`start_column` siblings — none of the
    five member paths exist on it, so it cannot collide either.
  - Locals bound under non-loc names are missed:
    `crates/vize_relief/src/relief/tests.rs` binds `SourceLocation::STUB`
    as `stub` and reads `stub.start.line` / `stub.start.column`
    (2 sites, test-only).
  - `Position` values escaping whole are counted where the `.line` /
    `.column` read happens, which a loc-shaped filter can miss: the two
    reads inside `internal_to_lsp_position`
    (`crates/vize_maestro/src/utils/position.rs`, receiver `pos`) are the
    only such sites today, reached only through `source_location_to_range`
    (0 references outside its own module at generation time).

## Reads per crate × member

| crate           | `source` | `start.line` | `start.column` | `end.line` | `end.column` | total | in test code |
| --------------- | -------: | -----------: | -------------: | ---------: | -----------: | ----: | -----------: |
| `vize_relief`   |        0 |            1 |              1 |          0 |            0 |     2 |            2 |
| `vize_armature` |        0 |            1 |              0 |          0 |            0 |     1 |            0 |
| **total**       |        0 |            2 |              1 |          0 |            0 |     3 |            2 |

## Every `line` / `column` read site

The line/column members are read so rarely that the sites fit in one table:

| member         | site                                                    | test code |
| -------------- | ------------------------------------------------------- | --------- |
| `start.line`   | `crates/vize_armature/src/parser/element/comment.rs:17` | no        |
| `start.line`   | `crates/vize_relief/src/relief/tests.rs:352`            | yes       |
| `start.column` | `crates/vize_relief/src/relief/tests.rs:353`            | yes       |

## Migration groups

### Group 1 — content reads moved to `Span::slice` (migrated by P1-3: 106 sites at P0-9, 0 remain)

The dominant consumer class by far: code that wants **the text a node
covers** — codegen re-emitting an expression, croquis capturing a binding
name, a lint rule inspecting raw expression text, a test asserting what the
parser captured. Each read used to pay for an owned `String` copied into
the node at parse time; the node now stores 8 bytes and the read is
`span.slice(source)` against the one authored source string (or, for
block-relative spans, against that block's text). Representative migrated
sites:

- `crates/vize_atelier_core/src/codegen/expression/generate.rs:39` — codegen reads the recorded expression text verbatim
- `crates/vize_croquis/src/drawer/template/components.rs:43` — croquis captures component/expression text into
  analysis products
- `crates/vize_patina/src/rules/script/template_scan.rs:193` — lint rule matches against raw expression text
- `crates/vize_atelier_vapor/src/transforms/v_on.rs:47` — vapor transform re-wraps an expression from the covered
  text (the owned copy the pre-span node stored is gone; see also group 3)

### Group 2 — line/column reads move to offset-derived rendering (3 direct sites + 4 known-missed, see limits)

Line/column exist only at diagnostic- or LSP-rendering time under Davinci:
derived from byte offsets via `vize_carton::line_index::LineIndex`
(`crates/vize_carton/src/line_index.rs:23`) at the edge that needs them — exactly how the
source-map `finish()` step and Patina's output layer already work. The
eagerly-stored `Position { line, column }` pairs these sites read today
delete with the type:

- `crates/vize_armature/src/parser/element/comment.rs:17` — the **only** production read: seeds
  `parse_vize_directive` with the comment's start line; becomes a
  `LineIndex` lookup from the span start (or a plain offset if the
  consumer is offset-ready)
- `crates/vize_maestro/src/utils/position.rs:102` — `source_location_to_range` converts stored
  `Position`s to LSP positions via `internal_to_lsp_position` (2 of the
  4 known-missed reads: `pos.line` / `pos.column`; the other 2 are the
  `stub` locals in the relief tests); its replacement
  is the offset → `LineIndex` path every live maestro collector already
  uses — and it has no callers, so see group 3
- `crates/vize_relief/src/relief/tests.rs:352` — relief tests pinning the 1-indexed line/column
  convention of stub locations; re-target to span offsets (the 1-based
  convention itself becomes a rendering-layer concern)

### Group 3 — delete outright

Structures whose entire job is carrying or converting the eager
representation. No slice or line/col replacement — they stop existing:

- `crates/vize_maestro/src/utils/position.rs:102` — `source_location_to_range` has
  0 references outside its own module; dead conversion code
- `crates/vize_atelier_jsx/src/span.rs:18` — `SpanMapper` eagerly expands oxc byte spans
  (`oxc_span::Span`, already `{ start: u32, end: u32 }`) into owned
  `SourceLocation`s for every lowered JSX node; under S0 the oxc span
  passes through unchanged and the whole conversion layer deletes
- `crates/vize_relief/src/relief/core.rs:120` — `STUB_LOCATION` / `SourceLocation::STUB`: the
  owned-string stub machinery for generated nodes collapses to
  `Span::new(0, 0)`
- `crates/vize_relief/src/relief/expressions.rs:33` — `SimpleExpressionNode` stored `content: String`
  **and** `loc.source` duplicating it; since P1-3 the node keeps one span
  next to `content`, and every group-1 site that cloned `loc.source` to
  build another node (e.g. `crates/vize_atelier_vapor/src/transforms/v_on.rs:47`) slices on demand instead
