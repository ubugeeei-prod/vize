# Original Vue 2 interpolation document: private consumer plan

Private design for #6842/#6882, pinned to literal actual main
`6987c523ecd2bee6e8489e2973e4b7499ce25018`. The first private freeze used
literal `c34b3d7a98`; audited expression/provider/dialect blobs are unchanged.
Root authorized design only;
implementation requires root review. No source implementation, build/install,
public PR, queue entry or manual campaign is included during the 0.431 release.

## Genuine available owners and boundary

Borrowed provider #7726 actually merged signed `264c7d0740` with ten API
laws, nine compile-fail examples, a positive example, unchanged seven Vue 2
and five Vue 1 whole pinned oracles and protected all-104 acceptance.
The [provider plan](./2026-10-04-vue2-original-expression-doc-plan.md) keeps
its historical source/failure records; terminal #6842 comment 5976281003 and
#6882 comment 5976281152 supersede its publication-stage pending status.

The proposed consumer covers exactly one sealed original Vue 2 `TextView`:
literal default `{{...}}` framing, its admitted original FilterChain base
and every existing argument. It formats only supported AST windows; all
intervening authored bytes, including exact filter registry names, encoded
pipes, blank lists, trailing commas, delimiters and trim tails, remain verbatim.
It accepts no raw AST, string, caller SourceBlock/profile or modern carrier.
It supplies no whole-template, descriptor-selected SFC, historical File or
native runtime/compiler/default-formatting completion.

```rust
vue2_text_document<'o, 'a>(
    original: vue2::surface::TextView<'o, 'a>,
    allocator: &'a Allocator,
) -> Result<Vue2TextDocument<'o, 'a>, Vue2TextDocumentRefusal>
```

The private wrapper stores that same view and `Doc<'a>`, exposing only
`original()` and borrowed `document()`. No public constructor or transfer of
the Doc out of its original-view wrapper is planned. The SourceBlock comes
only from `original.child().component().block()`. Foreign/recovered/verbatim,
local-hole, encoded-delimiter and comment-syntax refusals remain at the real
L1 `text_for` admission seam; no fictitious duplicate validation pass is added.
Same-byte foreign controls exercise that actual receiver and cannot forge a
TextView. The caller allocator allocates Doc composition only, never AST authority.

## Tiny shared projection and proposed ownership

| File                                                          | Proposed change                                                                                                                                         |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/vize_glyph/src/native_doc/expression/origin.rs` (new) | Private enum borrowing either the existing RetainedExpression or a genuine NativeExpressionView; checked source/root/comment-coordinate accessors only. |
| `native_doc/expression.rs`                                    | Extract its existing root-to-Doc orchestration into one private sealed-origin callback; preserve public expression_document/result/error order.         |
| `native_doc/expression/source.rs`                             | Store that private origin, delegate the same source/comment/map checks, retain existing Context arity and checked gap behavior.                         |
| `native_doc/vue2_text.rs` (new)                               | Public sealed-view consumer, monotonic authored-span cursor and typed whole-Doc refusal.                                                                |
| `native_doc.rs`                                               | Separate additive module/export clause, preserving incoming SFC exports.                                                                                |
| Dedicated Vue 2 tests/fixture/capture hook                    | Whole Doc/output, custody and pinned semantic-preservation proofs only.                                                                                 |

Use a private reference enum shaped as `Retained(&'p RetainedExpression<'a>)`
or `Borrowed(&'p NativeExpressionView<'p, 'a>)`. It borrows the real sealed
facade, not a copied AST/source tuple or open trait. A temporary facade over
each original syntax may be reborrowed for the shorter projection call;
`&'p Expression<'a>` never becomes `&'a Expression<'a>`. The projected Doc
contains authored text and arena composition, not AST pointers; its public
wrapper still borrows the original component/chain through the saved TextView.
No admitted witness, syntax owner, AST root, diagnostic or comment is moved.
No temporary parse, decoding, AST clone/normalization, reified syntax IR,
new whole-text/filter/header/body scan or extra tree pass is introduced.
The existing AST visitor's checked token/gap reads remain the same visit.

Context may copy that private reference enum for comment callbacks, retaining
the original typed comment iterator and existing checked comment list; no
boxed iterator or new callback index is needed. The AST visitor/array/object/
sequence files and current depth 16, L1 input 31 and L0 numeric 4096 limits
remain unchanged. The original actual grammar/profile/options and legacy
lexer fact are retained; preserving `010` or escapes grants no strict/runtime
early-error admission.

Read each original base/argument's complete EmbedSource span in existing chain
order. Require the same physical authored root, authentic content/profile,
complete map projection, SourceBlock containment and monotonic non-overlapping
operand windows inside binding.span. Invoke the same private Glyph root Doc
callback once per original operand; append source gaps before/between/after
them from the authentic block, finishing exactly at binding.span.end. No
filter spelling is inferred from argument count or AST callee shape.
For `upper ()`, the original registry key remains `upper `; name/order and
`f`, `f()`, `f( )`/final-comma distinctions all survive in authored gaps.
Unsupported nodes/depth, incomplete entity projection, invalid gaps/framing
or source joins refuse the whole Doc; original observations stay inspectable.

## Independent law and execution strategy

Use full nonzero Unicode roots such as
`前🙂<template>{{ &#38634; + 1 | upper () | 后缀(2 + 3, '後',) }}</template>後`.
Assert complete original child/ordinal/component, chain/base/argument AST,
admission-owner, root/source fat-pointer, profile/options, map and diagnostic
identities before and after construction, printing, refusal and unwinding.
Move the component before borrowing, then prove repeated short Doc borrows
leave its normal Drop-owned observations intact. Real same-buffer duplicate
and equal-byte foreign heap roots, different occurrences, repaired/inherited
recovery, verbatim, syntax/depth/unsupported-node and entity controls retain
their precise L1 or Doc refusal; no substring or partial-output assertions.
L1-hole/comment/recovery/encoded-framing controls assert the actual `text_for`
refusal and retained observations; they cannot produce a malformed TextView.
Use only reachable entity-map controls, retaining L1 SourcePreparation when
that is the real seam. Genuine stock borrowed-comment laws exercise the
private shared origin separately, without granting Vue 2 comment admission.

Expected full Doc trees (including every source slice, line, group and indent)
and complete printed byte goldens cover wide/narrow widths, LF/CRLF, encoded
operators/literals/whitespace, Unicode, NBSP trim, CRLF tails, exact filter
names, argument order, zero/blank/trailing lists and multiple filter stages.
Require whole-output idempotence; only the test reparses its actual output
through the original native entry. LF/CRLF narrow nested-division cases must
preserve the pinned scanner's SPACE-only slash look-behind. Existing infix
Docs break after the operator and keep its preceding separator as Space;
comments/regex retain their actual refusal rather than scanner replacement.
Existing retained-expression full Doc/output cases stay unchanged. Actual
compile-fail laws cover forged raw input/wrapper, owner/source/original-arena/
Doc-arena escape and owner move/drop while a returned Doc is live, with a
positive real TextView example. Catch-unwind tests retain original AST and
normal component Drop custody; no production panic/drop hook is introduced.

Future hosted Rust capture must provide actual complete rendered output,
joined to independent complete goldens before Node checks. The unchanged
seven Vue 2/five Vue 1 packets remain byte-exact. Additional pinned official
2.7.16 compiler and dev/prod VNode/ordered-filter-call checks compare original
and actually formatted complete source against their own complete compiler
goldens; generated render strings need not be byte-equal after intentional
formatting. Lookup/evaluation order is separate
from inner-before-outer invocation order. This proves formatter semantic
preservation only. Physical original slice/map custody is checked, but no new
source-map API or native compiler/runtime credit is claimed. Zero additional
parse/decode follows the actual sealed-provider-to-Doc call graph; source
counts/read-only inspection must stay distinct from executed fixtures.

## Coordination and held TODO

Formatter peer confirms no active Expr Context/source edits. Incoming #7693
`5eebcb1b` replays the frozen `e83f1db4` SFC exports/provider/laws and legacy
memmem restoration onto actual `6987c523`, plus a qualification-doc commit;
its expression provider blobs equal this actual main. It remains open and is
not an imported
baseline or dependency: this consumer needs only already-merged #7726 and
the existing Glyph Doc provider. Preserve its SFC/header/handler/conditional
targets; recheck literal main and leases before any later implementation.
The reused peer's exact first-freeze `60719f0d` review is clear for bounded
source readiness only; the explicit law refinements above grant no execution
credit. Root implementation review and authorization remain pending.
After frozen-plan peer/root review, implementation still needs authorization.
Future source/full Actions, immutable all-104 caps, protected queue and actual
merge remain required; release-flight restrictions remain in force. Historical
File/receiver, complete dialect grammar/registry, classic 2.7.16 ABI/runtime,
versioned SFC, other dialects and default replacement remain explicit TODOs.
