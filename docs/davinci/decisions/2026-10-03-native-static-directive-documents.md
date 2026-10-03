# Native static Vue directive documents

Date: 2026-10-03  
Owner: [#6847](https://github.com/ubugeeei-prod/vize/issues/6847)

## Decision

Extend the opt-in native Vue template Doc consumer from the delivered
static Bind/Prop heads to the existing L1 `DirectivePrefix::Full`,
`On` and `Slot` forms with nonempty `ArgSyntax::Static` arguments.
The previous binding slice actually merged in #7470 as signed `9bc6f5b6`.

The shared Vue grammar provides absolute spans for the full directive name,
argument and complete modifier run. Full heads project the original `v-`,
name, colon, argument and modifiers in order. Shorthands project their
original prefix, argument and modifiers. SourceBlock bounds and UTF-8 checks
cover every borrowed piece; original prefix and colon bytes must agree with
the typed framing. Glyph does not classify directive names or parse them again.

This accepts `v-bind:arg`, `v-on:event`, `v-slot:name`, custom full/static
heads, `@event` and `#slot` alongside `:arg` and `.arg`. Acceptance describes
source layout, without claiming semantic validity of directive names or
modifier combinations. Full forms without a static argument and all dynamic
arguments remain explicit refusals. Recovered or noncontiguous heads retain
the original parse observations and refuse document construction.

Attribute order, complete head spelling, modifier runs, values, quotes,
entities, JS/TS text, Unicode and authored multiline content remain borrowed
unchanged. Existing Doc grouping lays out opening-tag separators and indentation.
The formatter's default entry points and legacy fixture bytes are unchanged.

## Validation

Four projection laws check original pointer identity and reject forged prefix,
name, separator, argument, modifier, UTF-8 and out-of-block ranges.
Ten binding/directive integration laws check full flat/broken output,
Unicode/custom names, opaque values, order, fixed points and explicit refusals.
Seventeen existing document/layout laws retain plain-tag, recovery,
interpolation, source-custody, content and line-ending controls. Old event/slot
refusal examples now use dynamic arguments, while new positive laws exercise
their supported static forms.

The local proof compiles actual current L1 production source and selected
ordinary native Doc modules with Rust 1.98.0 and strict Clippy against pinned
retained dependency artifacts. It is scoped source proof, with no whole-current-
dependency or hosted-workspace acceptance claim. Canonical Glyph inventories
come from the unchanged repository generators. Fresh required/full/all-100
Actions and the protected queue must validate the published exact head.

## Remaining work

Dynamic arguments, full directives without static arguments, interpolation,
embedded formatting, other Vue dialects, complete product coverage and default
rollout remain unfinished. [#6882](https://github.com/ubugeeei-prod/vize/issues/6882)
stays open. No legacy-backed acceptance or formatter replacement is introduced.
