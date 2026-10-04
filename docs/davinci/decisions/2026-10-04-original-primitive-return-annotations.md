# Original primitive return-annotation design (2026-10-04)

Tracking: [#6849](https://github.com/ubugeeei-prod/vize/issues/6849) and
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879).
The [paired issue draft](./2026-10-04-original-primitive-return-annotations-issue-draft.md)
is unposted. This is a private design, with no Rust implementation, compilation,
runtime evidence, publication, queue admission or default migration.
The approved plan now underlies the separate [private provider source](./2026-10-04-original-primitive-return-provider.md); this design itself grants no executed provider or consumer evidence.

## Genuine source and prerequisite

This isolated `wt` starts from literal actual main
`b9be9065b872086726158b67d1b4e7b8dee78a14`. The existing normal function
walk still refuses every `function.return_type`. The original Program parser,
checked source/profile, actual function declaration, normal function child
scope, parameter/body declaration and reference events already exist.

The proposed predicate depends on the genuine typed-parameter provider
[#7756](https://github.com/ubugeeei-prod/vize/pull/7756), refreshed head
`1ee6e1d3ddd292898818d696d740ca8e9e4dd4a0`. Its private
`annotations::is_primitive_type(&TSType)` reuses the existing seven-keyword
match. That provider and its genuine diagnostic child #7758 form native Stack
#7759. Neither is actually merged on this design's base. Accepted execution on
an earlier head is not execution of the refreshed source or this proposal.

`native_typecheck` reserves the shared `walk/functions.rs` and
`walk/annotations.rs` through its actual delivery. No competing source edit is
made here. Implementation requires root and peer design review, explicit root
source authorization, and an authentic delivered dependency. Prefer fresh main
after the provider actually merges; if publication must precede it, use a
proper linear native Stack with the exact real parent, never a merge branch or
prose-only dependency. Do not alter an already healthy queued prefix.

| Actual source                                            | Existing authority and limit                                                                                                                                                                            |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `davinci/vize_l2/src/lang/js/file.rs`                    | `ProgramInput::checked` proves original admitted source, whole SourceBlock/profile/default parser options; FileProducer visits the Program once. Script and definition units are incomplete.            |
| `lang/js/file/walk.rs`                                   | The actual original statement event rejects setup/ordinary eligibility for every function before its normal declaration event; existing body statements and references retain their real lexical scope. |
| `lang/js/file/walk/functions.rs`                         | A genuine FunctionDeclaration supplies original return annotation, formal parameters, body and authored span. The old blanket return refusal is the only proposed production seam.                      |
| `lang/js/file/walk/annotations.rs`                       | The existing private primitive match returns only bigint/boolean/null/number/string/symbol/undefined. #7756 exposes a private boolean check; it adds no annotation row.                                 |
| `davinci/vize_l4/src/targets/ts.rs`                      | Completed whole-Module File projects its exact authored bytes plus the existing module suffix; original File/unit borrow and link sinks remain unchanged.                                               |
| `crates/vize_canon/src/corsa_bridge/original_program.rs` | Existing explicit checker authenticates actual source path/bytes/configured root/project and independent full LSP report; no default or unsaved-buffer claim.                                           |

Paths below `lang/js` are beneath `davinci/vize_l2/src/`.

## Bounded admission at the existing function event

Replace only `function.return_type.is_some()` in the existing function guard
with a checked `return_supported` boolean. A missing annotation remains true,
so every existing untyped JS/TS/JSX/TSX function and export policy is unchanged.
When an original annotation is present, require all of:

1. The existing event is in `Context::Unit`, with its actual named normal
   FunctionDeclaration and supported body; all other envelope guards remain.
2. The actual original parser profile is TypeScript Module, not definition;
   TSX retains the genuine original owning JSX File when used by consumers.
   Existing ProgramInput default-options/source authority remains mandatory.
3. The event's existing `exported` flag is false. This flag means a directly
   exported declaration; it is not a whole-program export classifier.
4. The original `TSTypeAnnotation.type_annotation` is one of the seven existing
   reference-free keyword variants.
5. Both original annotation span and inner type span pass `Walk::span`, which
   projects through the genuine ProgramReferenceSource to authored root bytes.

The check must happen before a supported function scope/parameters/body is
published. Keep the existing function binding/export partial facts and exact
whole-function UnsupportedSyntax refusal when a new annotation is unsupported;
invalid source projection retains the existing InvalidSpan issue. There is no
success path after any genuine File issue or interrupted unit.

A later original `export { f }` may still resolve a preceding admitted function;
no extra scan is added to reject it. Direct `export function f(): number` stays
incomplete. Default-export functions retain their current unsupported original
export path. Untyped direct exports retain their current complete behavior.

Seven keywords have no type/value references to resolve. Checking them at the
original function event needs no type namespace traversal, source slicing,
name guessing, secondary parser, Program.body rewalk or resident index. This
slice adds no public carrier, semantic row, SetupAnnotation, erasure receipt,
source buffer, pipeline stage or serialization. It makes no measured speed or
heap-neutrality claim; actual instruction caps remain the acceptance gate.

The following remain unsupported: void, never, any, unknown, object, literal,
parenthesized, union/intersection, array/tuple, named/qualified reference,
typeof/query, conditional/mapped/indexed types, type predicates/assertions,
generic/this return forms and every other non-keyword TSType. Async/generator,
declare/overload/missing body, type parameters, this/rest/default/optional/
decorated/pattern parameters, nested functions, unsupported body statements,
recovered parses, foreign source, Script/Unambiguous/definition/nondefault
profiles and unresolved body reads retain their current refusal boundaries.

An admitted return annotation grants no function-body type correctness. The
original external checker must still diagnose incompatible returned values.
A completed neutral File may have a nested SourceBlock unit, but genuine
VueSetup/ordinary eligibility stays refused by the existing statement event;
no whole-SFC compiler/runtime/template visibility is added.

## Independent authored laws planned before source publication

The following are planned laws, not tests run by this private design.
Freeze inputs and full expected observations before Actions; do not derive
expected issue vectors, diagnostics or authored coordinates from native output.

| Family                     | Required proof                                                                                                                                                                                                                           |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Exact old boundary         | Preserve `function f(value): number { return value; }` byte-for-byte as a new positive, removing only that exact obsolete entry from the old function negative list; every other input/assertion stays.                                  |
| Seven keywords             | Real TS and TSX observations admit each of bigint/boolean/null/number/string/symbol/undefined; original function/parameter declarations and value references have exact units/scopes/spans, with no added type references.               |
| Binding behavior           | Typed-return function uses real parameters, recursion, forward root reads and sibling shadowing; lexical parents and post-function references remain original. Existing typed-parameter laws remain intact.                              |
| Geometry/custody           | Unicode, CRLF, comments and a nonzero original SourceBlock retain the same Program/Function/annotation pointers, authored declaration/use/function spans and exact source allocation. Real TSX owner cannot be replaced by generic File. |
| Direct versus later export | Direct typed-return declaration is incomplete with exact original partial binding/export/issue; later `export { f }` resolves the genuine admitted function; untyped direct export remains unchanged.                                    |
| Other forms                | Every non-keyword return type, unsupported function/parameter/body/sibling and unresolved read keeps complete authored issue vectors; no newly supported primitive annotation hides a later failure.                                     |
| Profile/source             | Actual Script/d.ts/nondefault/unambiguous/copied-root observations retain original parser or ProgramInput refusal, without mutating a raw AST to fabricate a profile.                                                                    |
| Interruption/lifetime      | Existing real observer unwind points also exercise a typed-return input; interrupted units remain incomplete, no later declarations are visited, normal original owners survive until their actual drop. Keep all old unwind controls.   |
| Setup boundary             | A genuine once Descriptor/setup Program/File can be neutrally complete, while VueSetup and ordinary/native setup cursor refuse before consuming template children. No new SetupAnnotation or erase receipt.                              |

Expected first production diff: `walk/functions.rs` only, plus narrow existing
function test registration/obsolete-list entry and new return-case test files.
Reuse #7756's private predicate; do not edit variable annotations or add another
keyword classifier. Keep every owned file below 350 lines and retain actual
File/ScriptUnit layout. No unrelated Canon/Maestro production changes.

## Exact projection and diagnostic consumer follow-through

The L4 checker projection already retains every original byte, including the
annotation. It needs no production return-type printer or erasure. A bounded
consumer law slice should follow the genuine provider rather than claiming
full checker integration from neutral File completion. It may share one source
PR if source ownership and review allow; otherwise use a real dependent native
Stack and preserve a green bottom prefix.

Use the unchanged `project_program` and genuine `project_jsx_program` APIs to
check complete authored output, all span links, both UTF16 endpoints and
NoLinks refusal. Independent TS6 diagnostics for correct and incompatible
returns must match frozen expected complete vectors; configured Canon TS7
checks must retain the entire raw report, related information, authored spans,
actual nondefault configuration/path/project receipt and process shutdown.
A generic JSX File must remain refused where the owning JsxFile is required.

Small inputs can include `function read(value:number):number{return value;}`
and `function read(value:string):number{return value;}` under the actual
strict configured project with its original `moduleDetection: "force"`. These
local-only `.ts` fixtures have no export/import Module-goal proof; strict and
module:ESNext alone do not authorize `check_original_program`. A genuine `.mts`
or later original export is a separate eligible control. Their distinct
expected results and actual root tsconfig are authored before execution. Unicode/non-BMP/CRLF, missing-return and later unsupported
unit controls must be included where accepted source grammar genuinely permits
those cases. Preserve all old TS/TSX source/map/config/identity negative laws.
No test may call a legacy generator to grant native authority; dev-only
independent diagnostic oracles remain permitted.

## Decisions, delivery and unfinished work

The implementation decision must be posted on both #6849 and #6879 and kept
in this companion plus the central decision in the same genuine source change.
The current issue draft is unposted; this design is not a docs-only PR.
Root and existing peer must review the frozen source/laws before publication.
Use existing affected source-built Actions and the protected complete queue;
no local Cargo/install/build or additional manual full campaign.
Fresh exact-head source acceptance, full protected runtime, all unchanged
100 level plus four formatter instruction cases measured three times with
pinned ceilings/ratchets/holds, and literal signed actual merge are separate.

The history ledger and original fixture denominator are unchanged. A primitive
return provider alone closes neither #6879 nor #6849, grants no history fixture
credit, and proves no full TS type support, Vue function setup semantics,
runtime erasure, unsaved-editor snapshot/import graph, coherent configuration
ABA or measured same-diagnostic 10x performance. All product defaults remain
unchanged. Those TODOs stay explicit through provider and consumer delivery.
