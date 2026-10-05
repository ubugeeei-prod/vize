# Readable component hover contracts

Issue: [#8026](https://github.com/ubugeeei-prod/vize/issues/8026).

## Contract and bounded repair

The maintainer reports that components with many props have an unreadable hover:
each whole contract type is displayed on one line. The current generated-contract
producer deliberately compacts typed macro arguments and runtime declarations.
Both component tags and rewritten imported identifiers reuse that producer.
The native TypeScript quickinfo Markdown path is separate. This repair does not
claim to resolve generic component inference in #8015.

Keep the short contract byte-exact when its field fits within 80 columns. The
fragment helper returns a borrowed value without parsing or allocating there.
Only an over-width generated type is placed in a synthetic TypeScript type alias
and formatted through the existing optional Glyph dependency at 78 columns;
the outer contract contributes two spaces. Reparse only that formatted synthetic
fragment to extract the complete type annotation's AST span. Restore any formatter
RHS line break/indent and optional leading union/intersection separator, then add the outer
field indentation once. Use that same AST's quoted-type spans to leave raw
newlines inside template literals and continued strings untouched. Do not search for an equals sign or trim a semicolon from
inside a type. Preserve quoted property spelling with the formatter's existing
`quoteProps: preserve` option.

Formatter errors, unsupported fragments and builds without Glyph retain the
complete original type. This conservative feature-minimal fallback keeps those
builds working; wrapping without the formatting feature is not claimed. No source
program is formatted and no native query, serialization, pipeline stage or public
configuration is added. The component footer, Markdown fences, resolution policy,
unsaved-source handling and authored token ranges retain their existing producers.

## Authored complete corpus and public replay

`tests/_fixtures/differential/lsp/component-hover-readability/` freezes the request-
derived original many-prop component, a short control and their importing parent
before product execution. Whole references retain all props, emits, slots and
model types, nested optional properties, long generics, a generic callback and
quoted punctuation (`;`, `=`, commas, brackets and angle brackets), escaped
quotes and a multiline template-literal type whose raw line content stays exact. The full
80-column two-space contracts are authored from those declarations; they are not
captured from current Vize output. An independent installed Oxfmt 0.63.0 check was
used only to review the authored alias layout; Vize's pinned formatter is 0.60.0.

A new 13th shared legacy corpus case compares both complete component-tag hover
responses, preserving the original twelve cases and giving zero native migration
or fix-history closure credit. Whole Rust controls use the same original files
and complete objects. The separate real-stdio test compares all six whole hovers:
import binding, script reference and template tag for both components. It checks
input conservation, retains all observed full objects and bounded elapsed times,
and uses the existing source-build receipt/raw-wire capture in Actions. Timings
are observations, not a speedup claim or a new budget.

The same committed RPC fixture and oracle can be replayed against an installed
published CLI by supplying `VIZE_LSP_BIN` and the existing native TypeScript
runtime; `VIZE_LSP_REQUIRE_SOURCE_BUILD=1` remains mandatory in CI. Root owns the
supported post-release public-payload replay. A bounded review of the actual vendored parser identified a missing AST variant
in the initial source: plain no-substitution template types are `TSLiteral`
`TemplateLiteral` nodes, distinct from interpolated `TSTemplateLiteralType`. Add
that exact visitor span hook and a whole formatter/collector control, retaining
every frozen original input and expected response byte. The initial `0dd56c0f`
Actions cannot grant acceptance to its meaningful successor; fresh execution is
required. No runtime reference is re-recorded.

Actual source Check `37297395887` rejects only the new integration test's direct
`tokio` attribute (`E0433`): Maestro intentionally uses its own minimal runtime.
Use the existing `vize_maestro::runtime::block_on` facade for the same public
future and whole objects, adding no dependency. Production and frozen original/
expected/RPC bytes stay exact. The failed build and any partial hosted runtime
are historical evidence; this test-only successor needs fresh full source Actions.

Source Actions, unchanged protected
104 instruction probes/full Rust/original corpus, actual signed merge and release
publication are pending. This source record grants no runtime acceptance.

The verified issue author is `ubugeeei`, GitHub ID `71201308`, with recognized
noreply address `71201308+ubugeeei@users.noreply.github.com`. Supply that reporter
as Co-author on the meaningful source commit and PR; verify actual final literal
trailers and primary authorship separately.
