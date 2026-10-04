# Original selected setup runtime segments

Tracked in [#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

## Decision

`write_selected_setup_runtime_segments(&NativeSelectedSetup, &mut Writer<L>)`
appends the normally retained original setup body into the target's existing
writer. Only the authentic selected receipt supplies the SourceBlock and actual
current-unit annotation rows from the sole original Program declaration walk.
No external Program, File, source or annotation tuple is accepted.

The private source projection is shared with the existing full DOM setup
emitter. It validates every annotation's nonempty ordered extent, source-block
membership and full-root UTF-8 boundaries before any append. A second immutable
projection copies the original runtime segments directly with full-file links,
omitting only those actual annotations. No extra Writer, intermediate string,
AST/header/body/source traversal, parser, resolution, stage or serialization is
introduced. The sealed owner already certifies module eligibility; this emitter
does not repeat that policy.

The full DOM emitter retains its generated-name preflight, exact getter/setter
wrapper, statement separation and maps. The new segment API generates no names
or wrapper. Its target must separately establish collision, runtime, helper,
component and placement semantics. This is a genuine primitive setup Vapor
prerequisite; SSR can consume the existing full setup emitter directly and gains
no artificial DOM/Vapor target dependency.

## Laws and acceptance

- Genuine moved JS/TS owners retain the same whole original Program/source/File
  while segments append into a preallocated existing Writer with exact caller
  prefix links, independently authored segment coordinates, helper order and
  Recorded/NoLinks byte equality. Unicode, annotations, comments and semicolon
  omission remain exact original source facts.
- Malformed private annotation controls reject overlap, order reversal, empty
  and reversed spans, outside-block extents and split UTF-8 boundaries. These
  controls test validation; they do not manufacture public setup receipts.
- Actual `Object`, `__value` and `__v_raw` declarations keep the full DOM
  wrapper's collision refusal while the pure original segment provider grants
  no generated-name or wrapper authority. Imports, arrays, wider TS annotations,
  calls and mixed ordinary/setup sources cannot mint the sealed input.
- Complete typed compile-fail examples refuse raw SourceBlock and generic
  borrowed VueSetup substitution. Existing owner/drop privacy laws remain intact.

Independent read-only review found no custody or lifetime blocker. No local
Rust build/install is performed. Configured exact-head Actions must execute the
new laws and existing complete DOM/native map/runtime captures, followed by the
protected full suites and all 100 unchanged three-run instruction ceilings.
Actual candidate and literal merge are separate required receipts.

## Accepted prerequisites and unfinished work

Both genuine selected setup layers in native Stack #7664 are actually accepted:
envelope #7658 merged `c3c0d044` at 2026-10-03T23:44:10Z and DOM #7663 merged
`85d3afe2` at 2026-10-04T00:14:11Z. Protected Checks 37161916241 and 37163446627
passed full Rust/tooling/differential and fixed all-100 three-run gates; complexity
was 5761 in each run. DOM retained all six complete current-source modules/maps
and pinned Vue initialization, explicit mutation/render and unmount observations.
[The terminal receipt](https://github.com/ubugeeei-prod/vize/issues/6840#issuecomment-5974928214)
keeps prior failed sources and candidate history separate from acceptance.

This provider adds no new template/runtime admission, numeric collection type,
For loop, alias output, target/default migration or compiler fix-history credit.
Vapor's actual whole setup component, raw maps and production runtime remain its
own consumer obligation. The separately owned nested interpolation provider and
exact original DOM counterexample require their own current-source acceptance.
