# Directive call expression width

Issue: [#7876](https://github.com/ubugeeei-prod/vize/issues/7876).

The actual signed main `6d26d84b4240e9356cf5078b6d277e181d311a42`
contains the delivered attribute-prefix and continuation-prefix repairs. Its
original deep call still formats the expression as a standalone `void (…)`
program, so the expression's line-width calculation omits the indentation at
which its value is emitted. The original Edit guard also loses its authored
multiline quote layout. The native formatter remains Unsupported for all
sixteen planned shared cases: handled 0/16, equivalent 0/16.

Keep the existing single parse and formatter/parser/core identities. Extract
the expression implementation in a move-only commit. For a retained direct
CallExpression only, supply the actual attribute-value indentation budget to
the same formatter. A shape-checked AST-in entry keeps the real Program,
ExpressionStatement and UnaryExpression parent chain, including comment spans.
The existing quote writer owns those formatted lines and its literal state
keeps raw template-literal continuations exact. All other expressions, Vue 2
filters, v-for, leading line-comment and parse-fallback paths keep their
existing ownership. No extra parse, broad rewrite or new option is introduced.

The new `expression-print-width-7876` corpus keeps the complete original deep,
compact and Edit inputs in LF/CRLF form. Sixteen complete independent Oxfmt
0.63.0 references cover tabs, Unicode, argument/leading/trailing comments, raw
template-literal content and two widths. The authoring reference is bound to
its exact provider entry hash. All 64 original/reference stock Vue 3.5.35
observations agree for DOM, SSR, tooltip arguments and actual guard/click
effects in two states. Public API tests require whole bytes and changed flags
on three passes; public CLI tests require all 80 complete check/write streams
and file bytes. Hosted stock tests must compile the actual formatted output.

The original sixteen-case width corpus, shared sixteen-case formatter manifest,
original thirteen-case audit and historical 300 references remain byte-exact.
A closed, hash-qualified current adapter changes only the original deep
call's current shared comparison; its old mismatch stays visible. The original
deep/prebroken and compact API cases receive the same independently authored
current output after whole original corpus/input/partial-output pins. Existing
continuation qualification remains separate and unchanged.

The authenticated unpublished 0.435.2 source CLI was built at release head
`212692e21c618cc36e8281f041e2aa704c46eb0c`, whose formatter/CLI source and
dependencies exactly retain source cut `6d26d84b4240e9356cf5078b6d277e181d311a42`
apart from version-only metadata. Its successful producer job
[113111721791](https://github.com/ubugeeei-prod/vize/actions/runs/37715578811/job/113111721791)
supports the before reproduction: 60 complete commands over the original
twelve references fail ten cases, with the Unicode and wide-call controls
matching. It grants no installed release or modified-source runtime credit.

Required: exact-head hosted source/native and whole differential qualification,
all unchanged instruction ceilings, root-owned protected queue and actual merge,
then supported installed release verification. Inline-child/closing-tag width,
other expression shapes, native formatter admission and the full #7876 scope
remain open. Preserve every original fixture and failed observation.
