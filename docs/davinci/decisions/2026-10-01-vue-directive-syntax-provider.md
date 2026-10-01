# Native Vue directive-head syntax provider

Issues: [#6841](https://github.com/ubugeeei-prod/vize/issues/6841) and
[#6836](https://github.com/ubugeeei-prod/vize/issues/6836).
Paired decisions: [#6841](https://github.com/ubugeeei-prod/vize/issues/6841#issuecomment-5929325024)
and [#6836](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5929326936).

The existing `VueDirectives::decompose` skeleton becomes an actual provider in
`davinci/vize_l1/src/dialect/vue3/directive.rs`. The generic markup module keeps
`DirectiveSyntax`, `DirectiveName`, argument/prefix types and a narrow public
compatibility export. This slice adds no dialect references or policy wiring to
the generic lexer and profile implementations. The current Vue grammars share this directive-head spelling;
version capabilities remain separately owned by their dialect modules.

## Contract and admitted syntax

The hook takes the complete raw attribute head and its absolute UTF-8 byte
offset. Its result is `Result<Option<DirectiveName>, DirectiveNameError>`:
`Ok(None)` means a plain attribute, while recognized malformed directive heads
retain their source evidence in `Ok(Some(_))`. Offset overflow is an admission
error, including for a plain attribute whose source range cannot be represented.
The hook does not validate an offset against an authored source it was not given.

Full directive names exclude `v-`; shorthand names are empty at the prefix.
Arguments and modifier runs use source spans, with dynamic arguments excluding
their brackets. Empty modifiers and recovered unclosed argument bytes remain
available for the existing lexical diagnostic layer. The hook recognizes the
logical full name `pre` even with modifiers, arguments or malformed argument
heads; matching the literal entire attribute string to `v-pre` is insufficient.
This provider does not itself activate verbatim tokenization or change a product
route.

The native boundary scan retains typed delimiter order, nested brackets and
parentheses, quote escapes and nested template interpolation without allocating
or recursing. Adjacent identical delimiter kinds share a scalar count, admitting
the existing 20,000-bracket nesting witness. The initial allocation-free provider
admits at most 64 simultaneous distinct delimiter/quote runs, including the outer
argument delimiter; a more complex nested head returns `NestingLimit`. That is an
explicit restriction relative to the current unbounded legacy scanner, and the
provider is not complete Vue syntax. HTML attribute boundaries still delimit
recovery inside unfinished dynamic arguments, including unfinished literals.

## Validation and preservation

Independent coordinate goldens pin full names, all four shorthands, Unicode
absolute offsets, argument/modifier spans, malformed heads, logical `pre`, source
overflow and both sides of the run admission limit. A separate dev-only Armature
parser AST oracle compares admitted names, argument content/staticness/spans,
modifier semantics and malformed recovery; its verbatim subtree tests validate
the existing `pre` policy. It is not a pair of tokenizer event recorders and is
not a normal dependency or a native implementation shortcut.

The relocation has a pure rename commit and a TypeScript replay script with
byte-preservation, idempotence and conflict-before-mutation laws. The L1 skeleton
ratchet decreases from two to one. Normal/build reverse dependencies remain
forbidden, and no pipeline stage, serialization, legacy result bytes, corpus
fixture source or numeric instruction ceiling changes. Exact-head Actions and
protected merge-queue validation are required before actual merge.

## Remaining work

- Remove the artificial distinct-run admission limit through a dialect-owned
  parser or explicit scratch strategy in actual file context, while preserving
  ordered malformed recovery and the allocation contract. Do not silently fall
  back or classify unadmitted directives as plain attributes.
- Integrate real logical `pre` decomposition with the native surface builder's
  callback timing and verbatim suppression, with subtree parity tests.
- Factor the existing lexer directive-prefix dispatch through the actual dialect
  syntax dependency when converging the duplicate lexer state machines; the new
  decomposition hook alone does not establish complete generic lexer isolation.
- Implement typed directive Shape dispatch and JS/TS language/grammar selection.
  A recovered `Dynamic(Span)` alone does not prove balanced valid syntax; typed
  embed admission must also consume lexical diagnostics and the file context.
- Complete other Vue dialect syntax, file descriptors, conversion pattern/core
  isolation and the product fix-history/native acceptance gates. This bounded
  provider closes none of those roadmap issues and switches no product route.
