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

The PR run for head `4fa2eb7c14fdffd7af1ebf9df18c62f71ca76fcb` actually
executes synthetic producer `bfe69ba4f749fd359958c5a6858dc8e34423c9c8`.
The primary API verifies its parents are actual main `a5384b78c94ff8b11c111dcfce00721a9b60711b`
and that PR head, with their complete producer/head tree identical at
`bd6aa03bc708e2acf221597dc8f81285285df688`. This retained PR-tree producer
passes all 80 CLI processes and 160 stock runtime observations in
[job 113123610247](https://github.com/ubugeeei-prod/vize/actions/runs/37719454793/job/113123610247),
including both original Edit cases and all comment controls; all four affected
Rust workers pass. Its CLI build receipt binds the synthetic producer, so the
result supplies no direct public-head or actual-main execution credit. Its
tooling failures identify source-witness custody after
the extraction and the required attribute field. Retain the complete original
2281-byte attribute test owner under its original SHA, verify all five complete
law bodies while allowing exactly one new false field in each of the two
struct literals, and retain the ten unchanged script law hashes under the
exact extracted owner. All historical references and audit pins remain exact.
The canonical SSR sweep has zero differences but its observer rejects 44366
files against the required 44367; keep that requirement and investigate the
missing witness. These partial source results grant no whole-head, protected
or installed acceptance; the corrected successor requires fresh Actions.

The corrected source head `6721ec0c7f4873a42039b29534e950bc9d7c0e6a`
passes all required PR checks, sixteen whole references, API fixed points,
CLI80/stock160, the original history and all 44367 canonical inputs. The old
SSR omission does not recur; its I/O cause remains unproven. Protected candidate
`59bb0e5467e711e7a4c7fc489c7b3748dd4d5093` is removed after its complex
formatter benchmark measures 245269 against the unchanged 244347 ceiling in
three identical runs; all 100 level ceilings hold. The original complex input
has no direct call in a directive value, yet attribute parsing computes an
unused expression budget for every attribute. Pass the existing physical
attribute depth and defer the multiplication, subtraction and clamp until the
retained direct-call AST branch. Preserve the exact value-indent formula,
single parse, real ancestors, all outputs, original inputs and every ceiling.
Independent review finds no semantic or depth blocker. Performance success
requires a fresh three-run hosted measurement of all 104 original ceilings,
fresh source controls and a new protected candidate before actual delivery.

The deferred-budget successor `844834c2af59d4e6f67e284ccca23e9e16880c79`
measures 245172 against 244347 in three identical hosted runs: a reduction of
97 still leaves 825 instructions over the original complex-template ceiling.
All 100 level ceilings continue to hold. Avoid repeated Unicode trim scans
only when both expression edges are graphic ASCII bytes (33 through 126),
where the original trim is exactly a no-op. Empty, whitespace, every control
including DEL, and non-ASCII edges retain the original trim predicate. Inner
raw literal bytes, sequence ownership and the existing program-output trim
remain exact. Independent review confirms these slice and classification laws;
performance and successor runtime qualification still require fresh Actions.
