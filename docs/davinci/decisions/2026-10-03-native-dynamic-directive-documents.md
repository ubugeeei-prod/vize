# Native dynamic Vue directive documents

Date: 2026-10-03

Owner: [#6847](https://github.com/ubugeeei-prod/vize/issues/6847)

## Decision

Extend the opt-in native Vue 3 template Doc consumer to complete nonempty
`ArgSyntax::Dynamic` heads from the existing L1 `VueDirectives` provider.
The already delivered Full, Bind, Prop, On and Slot prefixes share the same
checked source projection. Glyph does not add an expression parser, semantic
directive classification, provider facade or extra pipeline stage.

L1 retains the original interior argument span. The consumer requires its
adjacent opening and closing brackets, typed prefix/name/colon framing and
modifier run to cover the complete original head in order. Checked arithmetic,
SourceBlock bounds and UTF-8 boundaries guard every borrowed piece. The Doc
borrows the original brackets, expression, modifiers and complete attribute
value without decoding, reparsing or rewriting them.

Opening-tag separators and indentation use the existing Doc printer. Attribute
order, opaque JS/TS text, quotes, entities, comments, content and authored
multiline bytes keep their existing contracts. Empty, unterminated,
noncontiguous or recovered heads refuse construction while the caller retains
the original parse observations. Directives without arguments and interpolation
retain their refusals.

## Validation boundary

Two added projection laws check pointer identity for all dynamic head pieces
and reject forged brackets, UTF-8 splits, modifier offsets and arithmetic
overflow. Eight added native integration laws cover complete flat/broken
output for every prefix, nested arguments and string/template boundaries,
Unicode/entities/comments/multiline values, generated CRLF, mixed-head order,
fixed points, refusal observation preservation, the inherited directive nesting
capacity and foreign equal-byte heads.
The existing static and plain-template laws remain controls; their dynamic
refusal fixtures now exercise empty arguments.

Local validation checks Rust formatting, diff hygiene and the unchanged
canonical inventory generators. Hosted affected Rust Clippy/tests validate
the published exact source; the protected merge queue validates the full
workspace and instruction ceilings. No historical retained build artifact or
duplicate manual full campaign provides acceptance credit.

## Remaining work

Full directives without arguments, interpolation and genuine embedded
formatting, other Vue dialects, SFC assembly, public formatter options and
checked span-edit integration remain unfinished. Default formatter replacement
waits for [#6882](https://github.com/ubugeeei-prod/vize/issues/6882) and complete
native corpus acceptance. Existing product routes, legacy fixtures, budgets
and oracle policy remain unchanged.
