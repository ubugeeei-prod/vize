# Original historical Vue expression documents: borrowed provider and successor plan

This records the read-only readiness plan for #6842 and #6882 after actual
Vue 2 custody merge #7690, followed by the reviewed original borrowed
provider. Root authorized this bounded provider for independent publication
after complete source/law review. Hosted execution and protected acceptance
remain pending; native formatter and historical runtime are unfinished.

## Literal source and delivered lower custody

The isolated `docs/vue2-native-formatter-audit-20261004` worktree was created
from actual signed main `da66dc241cb7e6ad25fc52fbb4a1b2c1effec9e5` and fast-forwarded
to literal remote main `1d91f0aed0ac7496e5f334801f49afb598215332` for review, then
to actual main `c13900eff34eccab876f449f69216d385d23835c` before the private
plan freeze. The authorized provider was created from that literal head and
then replayed onto actual main `7f7b63122456fd066c86bcab7c281c9c6c9d389a`
before production/law review. After the first hosted test-compilation failure,
the stable message-borrow correction was replayed onto literal actual main
`fcf8f9e5960efaca1c4626cd6748b38f8db28abd`; the reviewed production blobs
remain identical, and incoming central decision clauses are preserved.
The provider/parser/historical sources remain
unchanged across this refresh; incoming original attribute-value, Document
entity and delivery-control changes are preserved without being consumed.
Audited parser, L1 Vue 1/2 embeds, L2 expression/interpolation APIs, Glyph
expression Doc, and pinned Vue 2 reference bytes have no intervening source
changes. Incoming original JS primitive-initializer facts and scoped SSR
changes are preserved; they supply no historical File/interpolation API.
#7690 actually merged at 2026-10-04T01:42:27Z; its exact source and protected
candidate passed the existing 26 native Vue 2 laws, seven compile-fail
examples, unchanged seven pinned 2.7.16 compiler/dev/prod runtime oracles,
and all 100 level plus four formatter protected caps. That terminal receipt
belongs to the lower custody slice, not this successor.

Existing `vue2::surface::ComponentParse::text_for(TextChild)` returns a
sealed `TextView<'o, 'a>` over the original component, physical CST child,
callback `TextBinding`, and admitted `FilterChain`. Its existing monotonic
binding-start lookup is O(log n); there is no additional callback index.
Recovery, verbatim mode, original source identity, occurrence, token framing,
and local/global boundaries remain prerequisites for this custody.

## Exact existing expression ownership

Both historical versions retain `NativeSyntax`, not `RetainedExpression`.
Vue 1 `TextBinding::syntax()` returns its `Option<&NativeSyntax>`; Vue 2
`FilterChain::base()` and every `FilterInvocation::arguments()` expose the
original once-parsed `NativeSyntax` values. The latter retain their ordered
filter names, whole invocation spans, argument sources, decode maps,
parser observations, comments, diagnostics and holes. The existing callback
calls `prepare_text_value` once on its complete raw content. It decodes Text
entities there, then slices/trims the existing EmbedSource; those existing
map slices are not another decode. For a fully admitted chain, the base and
each nonempty admitted argument reach the stock parser exactly once after
existing resource guards. Structural list refusal may retain an earlier
base parse but never starts argument parses before the whole list validates.
The proposed borrow and Doc add zero parser or decoder calls; this is a
source-flow requirement, not an executed successor-count receipt.

The pre-provider `NativeSyntax::expression()` returns `Option<&Expression<'a>>`
with the lifetime of the owner borrow. Its original `EmbeddingObservation`
selects the existing wrapped root through `shapes::expression`, checking the
single stored statement and outer parenthesized expression. It does not
issue a public sealed borrowed admission witness on the audited original main.

`NativeSyntax::into_expression(self)` consumes the original syntax owner.
The stock parser handoff pops its stored statement and allocates the moved
inner root in the original arena before returning `ExpressionObservation`.
`RetainedExpression::admitted_expression()` then returns the existing
private-origin `AdmittedExpression`. This is an authentic consuming API,
but applying it to historical bindings would dismantle their original
chain and root custody. Do not clone/reparse the AST, convert to a modern
carrier, or replace parser admission with `.expression()` plus a hole flag.

## Lowest ready provider: borrowed parser authority

The following original proposal is implemented and source-reviewed for
publication; hosted and protected execution remain pending:

```rust
EmbeddingObservation::admitted_expression(&self)
    -> Option<AdmittedBorrowedExpression<'_, 'a>>
NativeSyntax::borrow_expression(&self)
    -> Option<NativeExpressionView<'_, 'a>>
```

The stock witness must have private fields borrowing its actual
`EmbeddingObservation` and its actual root. Its root accessor returns
`&'o Expression<'a>`, preserving the shorter owner borrow; it must not
invent an arena-lifetime root. Construction uses the existing constant-size
root selection and original goal/hole only. Accessors borrow original
content, source type, parser options, parser content/container spans,
comments, diagnostics and the stored `has_legacy_literals` lexer fact.
No caller AST, numeric window, boolean admission status, replacement arena,
source string or profile may mint the witness.

The L1 facade holds the original `NativeSyntax` borrow and authentic stock
proof. It uses the existing Coordinates for decoded/authored spans,
original grammar/source/profile, full content identity, comments and
normal diagnostics. Wrong shapes and all local holes remain unadmitted.
Neither owner is consumed or changed. This adds no allocation, AST walk,
parse, decode, wrapper, callback index or resident pipeline stage.

Root authorized only the additive stock parser `embedding/borrowed.rs`
witness and re-exports, L1 `embed::syntax/borrowed.rs` facade and re-export,
and focused independent laws. They were privately implemented in the new
`feat/l1-borrowed-expression-authority-20261004` worktree from literal main
`c13900eff34eccab876f449f69216d385d23835c`. No Vue 1/2 callback storage,
scanner, CST, selected-SFC, body, registry or Glyph projector code changes.
The stock proof preserves the shorter root borrow, and L1 also requires
exact content fat-pointer, original source type and wrapper-prefix joins.

Ten concrete API laws cover exact root/content/grammar/profile/options,
full Unicode/nonzero-root decode maps, typed comment/diagnostic addresses,
and original normal Drop custody. Real compile-fail examples cover
witness forgery, raw-AST substitution, owner/source/arena escape and moving
an owner during its borrow. Nine compile-fail examples and one positive
public-API example are written; their actual execution remains pending. Moving owners before borrowing, short reborrows,
same-buffer duplicate parses, cooked entities, wrong shapes and retained
local holes supply distinct positive and negative controls. Existing
consuming observations and historical whole oracles remain unchanged.

## Genuine dependent product consumer

After the actual borrowed provider is available, propose one opt-in Vue 2
interpolation Doc entry, not whole-template or selected-SFC formatting:

```rust
vue2_text_document<'o, 'a>(
    original: vue2::surface::TextView<'o, 'a>,
    allocator: &'a Allocator,
) -> Result<Vue2TextDocument<'o, 'a>, Vue2TextDocumentRefusal>
```

The returned wrapper retains that actual sealed child view and its Doc.
The SourceBlock comes only from its original component. A caller allocator
may allocate Doc nodes; it does not become authority over the parser AST.
Glyph's existing retained-expression API remains unchanged. A private
original-source enum can share its current immutable bounded AST Doc
projection with the new authentic borrowed facade; no public raw-AST tuple,
open trait, reconstructed retained owner or legacy formatter is needed.
This source change requires coordination with the native formatter lane.

Project each original base/argument expression window in chain order, and
append every intervening authored source gap verbatim. The finished cursor
must cover the complete original interpolation, including delimiters,
leading/trailing whitespace, authored or encoded pipes, exact registry
name bytes, parentheses, blank lists and final commas. Check every mapped
window for monotonicity, non-overlap, full SourceBlock containment and
complete entity boundaries. This uses the already-selected operands;
it must not scan or parse filters again. Unsupported AST descendants or
projection failures refuse the entire Doc while retaining original syntax.

In particular, `upper ()` has the registry name `upper ` in the original
2.7.16 compiler. Names can be nonidentifiers. Do not trim/camelize that
name or represent it as a JS callee. `f`, `f()`, `f( )` and a final comma
cannot be reconstructed from argument count. Preserve their source gaps.
The first consumer changes only proven formatting inside the existing
base/argument AST windows; filter punctuation and spacing stay authored.
Vue 2 comments remain the current typed `CommentSyntax` refusal, including
encoded comments. No authored JS-comment formatting is claimed. The
provider must nevertheless retain comments/diagnostics on all observations.

Current Glyph expression Doc supports its bounded atom, parenthesis, unary,
binary/logical, ordinary member/call, conditional, array, sequence and
static explicit-object families with its unchanged depth-16 bound.
Regex/template/optional/type/spread and other unsupported descendants must
retain their concrete refusal. Vue 2's original native grammar is JS Expr
with its current compiler Module parser profile, not a newly chosen
formatter profile and not classic render-function grammar. Its existing L1
wrapped-input admission is 31 conservative units including generated
wrapping. The shared L0 safety guard retains nesting/speculation bounds of
31 and the 4096-byte numeric-token bound. These are separate from Glyph's
Doc depth 16; no bound may increase and no full JS/Vue 2 grammar is claimed.

Required hosted consumer proof: whole expected Doc/rendered output across
width and LF/CRLF options; idempotence; exact original AST/root/map/source
identity; Unicode/nonzero complete-file offsets; cooked entities and whole
entity refusal controls; exact filter names/order; zero/blank/trailing-comma
lists; foreign/recovered/verbatim/local-hole refusal; and decoder/parser
flow showing no additional parse or decode. The unchanged whole pinned
2.7.16 parser/compiler and dev/prod runtime corpus must still pass. New
complete official compiler/VNode/ordered-filter-call controls can establish
semantic preservation of original versus formatted source. They do not
establish native compiler or runtime execution.

A provider and this consumer are real dependencies: branch the child from
the provider head, establish and verify a genuine native GitHub Stack, and
merge only exact-green contiguous prefixes through the protected queue.
The pending Vue 3 formatter Stack is not an imported baseline or automatic
dependency. A later Vue 1 escaped-text Doc consumer can reuse the actually
merged borrowed provider in its own slice, preserving all-pipe, CR/raw/once,
recovery and encoded-framing refusals. It is not already implemented.

## Exact L2/File/runtime gaps

The shared Vue Descriptor rejects `options.version != V3`. There is no
admitted historical selected-SFC producer. Existing
`NativeInterpolationInput` owns a modern `NativeInterpolationOperand` and
joins only `NativeTemplateComponent` with its actual `NativeChild`.
The body/root cursor receivers, pending File fact, completion state and
text operation likewise use that modern source authority. None accepts a
historical `TextView` or whole original Vue 2 FilterChain.

`JsExpr::from_retained_in` requires `&'a Expression<'a>` and allocates its
metadata in the original selected arena. The proposed historical borrowed
root is only `&'o Expression<'a>`. The Doc consumer cannot lengthen that
borrow or manufacture a genuine owned File. A future historical owner and
receiver must resolve actual whole-owner lifetime authority, source/profile
selection, original ordered child cursor/prefix/sticky interruption, every
base/argument map and AST, registry resolution, and all-complete attachment
before minting an attached NodeId. These are separate missing designs.
The existing L2 `VueFilterExpr::parse_in` rescans/reparses raw source and
keeps textual arguments; it is not this genuine original-chain receiver.

L4 currently declares only VueDom, VueServerRenderer and VueVapor runtime
families. Its modern DOM helper vocabulary/module signature cannot select
a Vue 2.7.16 classic runtime. The complete pinned primary compiler
`tests/_fixtures/reference/vue2/build.js.gz` emits
`with(this){return ...}`, `staticRenderFns`, `_v(_s(...))` and nested `_f`
applications. The pinned runtime installs `_f = resolveFilter` and calls
render with the original Vue render proxy and `$createElement`.
`resolveFilter` uses `$options.filters` through `resolveAsset`, including
exact local registry keys, camelized/PascalCase alternatives, prototype
fallback and missing-filter identity behavior. The generator nests
`_f("outer")(_f("inner")(base, innerArgs), outerArgs)`: JavaScript resolves
the outer callee before the inner callee/base, but applies the inner filter
before the outer filter. Future native runtime laws must distinguish
registry lookup/evaluation order from filter application order. This is a
source inference from the original generator, not a new runtime execution.
A classic runtime factory, render-scope binding, asset resolution, source maps and actual complete
native module/VNode/call-order dev/prod proofs remain missing. Compiler
Module syntax admission is not classic runtime early-error validation.

## Held status and TODO

The initial audit performed no implementation. After root authorization,
only the bounded stock-parser/L1 provider and its ten API/nine compile-fail
laws were written privately. Root and the reused dialect peer independently
reviewed exact frozen `6969cf2645c76a91a51cfd74621faffb02e533c0` and found no
production/API/lifetime/custody/fixture blocker. Root then authorized
independent publication. The reviewed Rust blobs are unchanged by the
publication-status update. Focused rustfmt, oxfmt, two pure-Node module-layout
controls and the complete source-qualified canonical Croquis check passed;
none executes the new Rust API laws or doctests. No local Rust build/tests
or npm install was performed. The first hosted exact-head `00e64f55c2` affected Clippy completed, but
archive/lib-test compilation rejected a test's `.as_str()` call as unstable
`str_as_str`. The law now borrows the original diagnostic message through
an explicit stable `&str` coercion, retaining its complete comparison and
actual record-address proof. Production Rust remains unchanged from the
reviewed source. No API law or new doctest executed on that rejected head;
Corrected head `2f5768edc8` actually passed all ten API laws, nine
compile-fail examples and the positive doctest; all four source Rust shards
passed 15,312 tests. Its final report failed because the two new test files
were absent from the reviewed storage inventory. The source-authority law
now uses two distinct live heap buffers from the approved L0 String owner,
preserving equal bytes and every foreign-owner/root/pointer assertion.
The reviewed test rows record custody alloc-Vec 1/4 and L0 String 2/0,
and historical alloc-Vec 1/3; all other counts are zero. No production
storage, scanner, policy, parser, map, historical fixture or instruction
ceiling changes. This correction needs fresh exact-head execution;
hosted source/full/protected acceptance and actual merge remain pending. Existing old worktrees are preserved. The
Glyph consumer and shared expression adapter still require separate
authorization; no such source edits are included. Reuse existing peers
without new agent threads.

The initial private design is paired on #6842 comment 5975679409 and #6882
comment 5975680336; frozen private provider records are #6842 comment
5975873462 and #6882 comment 5975874089. Record publication on both issues and
carry this companion and central link in its genuine source change; refresh literal
main and peer ownership first. Exact-head Actions and all applicable
protected suites/caps must pass and the real merge must be verified.
#6882 remains open, so the legacy/default formatter route stays unchanged.
Whole Vue 2/1 grammar, historical SemanticFile/versioned SFC, registry and
runtime, Vue 0/quirks/petite, product acceptance and default cutover remain
unfinished. Source/AST custody alone grants none of that completion.
