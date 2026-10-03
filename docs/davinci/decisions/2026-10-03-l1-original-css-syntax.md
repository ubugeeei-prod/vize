# Original CSS syntax and bounded scoped insertion authority

`vize_l1::css::StyleSyntax::observe` accepts only an original admitted descriptor
`StyleView`. It retains that checked `SourceBlock` and container index, then
invokes the pinned production cssparser 0.37.0 parser once over the actual block
slice. It never copies, normalizes, serializes or reparses a stylesheet. The
actual qualified-rule/declaration callbacks retain library `Token` and `CowRcStr`
values with complete-file byte spans, including decoded escaped class names.
Actual parser errors retain their CSS-local UTF-16 locations and tokens. A
non-CSS language is retained unparsed with an explicit profile refusal.

These actual external parser token values require normal destruction. Their
source-sized token/declaration vectors therefore use normally owned `alloc::Vec`
instead of the Drop-free arena vector. This adds precisely the reviewed storage
rows for `css.rs` and `css/parser.rs`; no existing storage ceiling is relaxed.

The smallest scoped receipt lends exactly one complete `.class` qualified rule
per original style block, with flat declaration values. Multiple original style
blocks can each provide a receipt. Selector grammar is checked against actual
parser tokens; whitespace between the dot and name is refused, comments are
retained, and escaped class names keep their actual original token bounds.
The insertion position is the end of the original class-name token.

At-rules, multiple rules, empty scoped blocks, pseudos, combinators, attribute
selectors, nested rules/values and functions (including escaped `v-bind`) do not
lend this receipt. Parser recovery at a missing rule brace is refused by checking
the parser's actual stop position against the authored closing boundary. Flat
quoted values require identity with the actual decoded token; unproven escaped
values are refused. Original blocks, partial actual rule/token data and real
errors survive every refusal. No API accepts caller-supplied source, tokens,
spans or completion flags as authority.

The pinned Vue 3.5.35 audit also proves that a returned QuotedString containing
`v-bind(color)` is not a transform-free flat value: its CSS and descriptor CSS
variables change. Comment markers inside quoted tokens can bridge descriptor
binding spellings even when CSS output stays literal. Actual returned quoted
tokens containing `v-bind` or `/*` therefore retain UnsupportedValue refusals,
their original token spans and partial declaration data. This is a conservative
token-family boundary, not whole-style scanning or a binding implementation.

This provider does not implement Vue scoping or claim property semantics. The
existing plain CSS consumer and typed scoped-style guard stay unchanged here.
Its dependent product slice must insert only generated scope attributes at
these proven positions, preserve every emitted original byte and map, attach
the same `__scopeId` to the complete native component and prove actual Vue DOM
scoping. Full CSS grammar, scoped selector families, `v-bind`, CSS Modules,
preprocessors and whole-product/history gates remain unfinished.

Validation laws cover original pointer/source custody, escaped and Unicode/CRLF
coordinates, actual declaration/token values, selector whitespace and shape,
partial real parser errors, external-profile refusal, and private constructor
custody. Actions and the protected immutable all-100/full suites remain required.
