# Original Vue 2 interpolation document: private consumer plan

Private design for #6842/#6882, pinned to literal actual main
`6987c523ecd2bee6e8489e2973e4b7499ce25018`. The first private freeze used
literal `c34b3d7a98`; audited expression/provider/dialect blobs are unchanged.
Root first authorized design only. Frozen peer/root design review is clear;
root subsequently authorized the bounded private implementation below. It
remains unbuilt and unpublished, with source review required before Actions.
No build/install, workflow dispatch, public PR, queue or manual campaign occurred.

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

Formatter peer confirms no active Expr Context/source edits. The plan audited
incoming #7693 `5eebcb1b`, a frozen `e83f1db4` SFC/provider/law replay onto
`6987c523`. It subsequently actually merged signed `aae579abe6`; private
consumer source replays onto literal actual main `7516c514bc`, preserving
those genuine SFC exports. Expression/provider/dialect blobs are unchanged.
This consumer needs only the already-merged #7726 and existing Glyph Doc
provider; file adjacency supplies no fake pending-provider Stack or historical
whole-SFC completion. Preserve its SFC/header/handler/conditional
targets; recheck literal main and leases before any later implementation.
The reused peer's exact first-freeze `60719f0d` review is clear for bounded
source readiness only; the explicit law refinements above grant no execution
credit. Root subsequently authorized private source only, after plan review.
The frozen plan review grants only the explicitly authorized private source scope.
Future source/full Actions, immutable all-104 caps, protected queue and actual
merge remain required; the private source-review hold remains in force. Historical
File/receiver, complete dialect grammar/registry, classic 2.7.16 ABI/runtime,
versioned SFC, other dialects and default replacement remain explicit TODOs.

## Private source and unexecuted acceptance

The isolated `feat/glyph-native-vue2-text-document-20261004` wt starts from
literal released `6987c523ec`, then replays onto actual `7516c514bc` after
formatter delivery; old wts and the reviewed private design remain intact. Production adds only the private sealed reference origin, factors the
existing root callback/Context source checks and adds the public view-owning
Vue2 Doc wrapper. All four AST visitors, public retained APIs/error order,
limits, historical callbacks/storage/scanners and pinned oracles are unchanged.
The complete authored cursor covers the exact binding span without filter
rescan or new decoding/parsing. Original fields are never moved or cloned.

Eleven Rust laws, eight real privacy/lifetime compile-fail examples and one
positive example are written but unexecuted. Full Doc trees, source/map/stock
record identity, foreign same-buffer/equal-heap roots, nonzero Unicode/nested
children, module lexer facts, ordinary Drop/unwind, genuine L1 refusals and
whole Doc refusal are explicit. The 48 authored print/compiler rows cover
LF/CRLF, zero/narrow width, historical division, entities/trim, blank/zero
lists and separate lookup/read/invocation events. Expected goldens contain
no fabricated native output or receipt. Full old seven Vue2/five Vue1 packets
remain byte-identical; this is source custody, not fresh execution credit.

A distinct composite action, narrowly selected on source shard one and
mandatory in merge groups, writes a fresh real Rust output packet and joins
it to the complete pinned compiler and bounded div/text runtime expectations.
It records actual source head separately from execution commit/run and retains
the raw artifact. The unchanged official compiler/runtime helpers authenticate
every original package byte. Ordinary oracle discovery explicitly skips its
native join without that capture; the dedicated source/merge hook requires
all 48 actual rows. No failed candidate may gain credit by later recapture.
No native historical compiler, File, ABI, SFC/default or full grammar is added.
Complete source/law review, then authorized exact Actions, all-104 protected
acceptance and actual merge are still pending; release hold lift does not
authorize publication of this privately held source.

## Private source review and inventory correction

The reused peer's complete read-only review of frozen `0c80ef034b` found no
production, lifetime, law, independent-golden or capture-provenance blocker;
it held that freeze only for the omitted owned Glyph consumer inventory.
The unchanged canonical direct Node generator now adds fourteen genuine rows
only to `formatter/vize_glyph.tsv`; its complete nineteen-file check passes,
with every foreign shard, policy, AST visitor and pinned historical oracle
unchanged. This is pure source preparation, not Rust or native-output execution.
The corrected exact freeze still requires root and final peer source review
before any build, dispatch, publication or queue. The original `0c80` omission
and unexecuted laws remain explicit; no acceptance transfers from that freeze.

## Draft publication and pending hosted proof

Root and the reused peer completed the entire corrected `2f47cb6c8c` source,
lifetime, eleven-law/eight-new-compile-fail/positive, full 48-row oracle and
capture-action review. The final exact peer receipt is source CLEAR only.
Root authorized independent draft publication and automatic source Actions.
The clean replay onto literal actual main `b9be9065b8` preserves every reviewed
production/law/fixture/oracle/capture byte and genuine SFC export, plus incoming
central decisions and all foreign inventory shards. This publication record
supersedes the historical private hold above; no Rust or native capture has
yet executed. No additional manual campaign is requested. Ready/auto-queue
requires fresh exact-head source proof; protected full suites/all 104 original
caps and actual merge remain required before delivery. The formatter history
issue #6882 is now closed; historical dialect formatting/runtime/default
completion remains independent and unfinished.

## Initial hosted source rejection and narrow correction

Draft #7761 first published `418d746e42`; automatic Check `37181475857`
rejects two real source details. `fmt-rust` resolves the new explicit-path
Origin module's plain test declaration to a nonexistent sibling `tests.rs`;
the declaration now names the existing `origin/tests.rs` explicitly. Hosted
Clippy also rejects the private append helper's needless explicit owner
lifetime; elision preserves the same short input borrow and public API.
Both complete original failure logs are retained. These two narrow source
corrections preserve all public signatures, runtime statements, laws, expected
bytes, original custody, action/oracle behavior and ceilings. Recursive direct
rustfmt source checking now resolves the real modules; no local Rust build
or test is run. This failed source supplies no native acceptance. Fresh exact
source Actions and all actual captures/laws/protected proof remain required.

Fresh `33fbb488cf` passes hosted fmt and Clippy, then rejects one test-only
parent pointer comparison with E0308: the original child accessor returns
`&Element` while its real CST surface stores `&Box<Element>`. The law now
compares the actual `&**element` address, preserving the same original physical
parent assertion rather than comparing a carrier. Every production/API,
input, expected output, other assertion, oracle/action and cap stays unchanged.
The complete rejected build log is retained; failed `33fb` carries no test or
native-capture acceptance. A fresh exact source remains mandatory.

Exact `ef9570aab1` executes all eight new compile-fail examples and the positive
Doc example, and its first tooling worker successfully captures all 48 genuine
native rows and joins complete compiler/dev/prod runtime expectations. The
initial raw packet binds source `ef95`, execution `8fa7a82ae6` and workflow
`37182189177`; it stays immutable and separate from independent authored goldens.
The same source's fourth worker rejects the shared workflow at 355 lines
against its unchanged 350-line limit. Removing only five structural empty
separator lines restores the bound with every nonempty YAML line, event guard,
step order, action, shell statement and script body unchanged. Existing length
laws, policy and all instruction caps are retained. This whole failed source
still grants no queue acceptance; fresh exact Actions/captures/full protected
proof remain required, without changing any native expectation or input.

The same complete `ef95` Rust execution retains two rejected new laws: valid
plain operands inside a once-decoded original callback hit SourceMismatch.
Actual `EmbedSource::slice_in` deliberately omits a resident map for an
identity-only piece while retaining its decoded parent buffer; no-map does
not imply authored/text pointer equality. The bounded correction keeps the
existing Retained-origin raw-pointer policy exactly, while Borrowed-origin
uses its authentic stock-owner/validated private EmbedSource proof after the
unchanged physical-root, block-coverage and full-span projection checks.
It adds no byte scan, parse, decode, allocation, source reconstruction or AST
walk. All original failing whole sources, expected outputs and custody
assertions remain unchanged. A genuine independently prepared stock identity
slice additionally proves its exact authored span, absent map/different text
pointer, original AST identity, complete output and foreign-root refusal;
the existing public Retained API still refuses that same real source. These
new assertions extend the existing private law rather than inventing an invalid
sealed view. The initial 48-row raw packet stays immutable. Full root/peer
review of this production correction and fresh exact hosted acceptance remain
required; failed `ef95` supplies no whole-source or queue acceptance.
