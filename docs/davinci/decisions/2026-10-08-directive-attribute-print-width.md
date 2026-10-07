# Directive attribute-prefix print width

Paired issue: [#7876](https://github.com/ubugeeei-prod/vize/issues/7876).

An already broken start tag can still put an entire directive value on one
line after its attribute name. The expression formatter receives the full
`printWidth`, so it can produce a single line whose attribute prefix and
actual SFC indentation then overflow that budget. The first binding in the
complete `Example.vue` report reaches 87 columns at width 80; the deep
`:title` report becomes 108 columns after its authored quotes are joined.

The existing attribute printer now measures the rendered prefix, indentation
and value. If a single-line expression attribute overflows, it places the
value and closing quote on their own lines. Private SFC base-depth context
gives these continuation lines their final indentation: the existing SFC
raw-line mask preserves a value starting below an empty opening quote.
Display width uses the already pinned workspace `unicode-width` dependency;
tabs count as configured indentation columns. No additional expression parse
or pipeline stage is introduced.

This slice keeps the existing start-tag break decision. It preserves literal
attributes, `v-for` grammar and already multiline expressions. The deep
reported call has 89 columns before its twelve indentation columns: its own
value line remains 101 columns at width 100. Internal expression reflow and
whitespace-sensitive inline child layout remain TODOs in #7876. The multiline
`Edit.vue` guard report also remains unresolved. This PR references the issue
and does not close it.

Two preceding move-only commits extract the existing attribute writer and
opening-tag parser into private modules. They preserve the existing test
sources and keep the changed production files within the 350-line limit.

The new differential corpus retains the complete original `Example.vue`
source and configuration, the complete deep prebroken source, and fourteen
compact/boundary/event/tab/Unicode/line-ending/bracket/literal/`v-for`/standalone
controls. Every input and full expected output has a SHA-256 in `corpus.json`.
Eleven complete outputs independently come from cached Oxfmt 0.63.0; the Auto
case uses its explicit CRLF reference because that provider has no Auto
option. Original Example output combines the issue-authored attribute layout
with independently formatted script bytes and retains its unresolved inline
content. The two deep cases preserve the reporter's complete authored quote
layout; the `v-for` and standalone references express existing grammar and
zero-outer-depth layout directly. These are distinct reference authorities.

Public API tests compare all sixteen full outputs over three passes and
assert `FormatResult.changed`; fifteen SFC cases also exercise configured
CLI check/write/check with complete unchanged/changed file bytes. The default
deep case is registered as an active shared formatter input with three CLI
passes and an explicit unsupported native route. Original historical sources,
captures, expectations, all 104 ceilings and native acceptance remain unchanged.

Local reference review used Vue compiler-dom 3.5.41: all sixteen original and
expected templates compile to identical render JavaScript after formatting
only that generated JavaScript's whitespace with Oxfmt. All 300 historical
API plans still load through their existing immutable source/receipt validators.
Those inspections grant no source-built API or CLI execution credit. Fresh
exact-head Actions, protected qualification and actual merge remain required;
root owns the queue and release.

Shared registration increases the finite manifest assertion from thirteen to
fourteen and admits only the new named pending authored reference beside the
existing named pending reference. Captured baselines keep their original
captured state; there is no broad pending-state exception. Pure validator
tests run locally; source-built shared CLI execution stays on Actions.
