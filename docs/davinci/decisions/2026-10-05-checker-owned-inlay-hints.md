# Preserve checker-owned variable type inlay hints

Issue: [#8004](https://github.com/ubugeeei-prod/vize/issues/8004).
Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8004#issuecomment-5993319002).

The original SFC and literal Vize configuration are retained under
`tests/_fixtures/differential/lsp/computed-inlay-hints/`, with hashes and the
reported Vue 3.5.41/version/environment. The tsconfig fixture reconstructs the
fields supplied in the report; omitted compiler options are not claimed as
original bytes. The frozen workspace lock already contains Vue 3.5.41, so the
fixture selects that exact installed package rather than the Playground's
different Vue release. It does not change workspace dependency links or locks.

The old reactive formatter substituted `_` for an unknown syntax-inferred type.
The actual LSP handler now obtains native variable-type hints through one
standard range request on the existing reusable editor session. The initialization
preference is the supported `inlayHints.variableTypes.enabled` field. Other
diagnostic/style preferences, ATA policy, configured project authority, readiness,
recovery and native request generation/cancellation ownership remain intact.

Pinned primary sources are TypeScript 7.0.2 commit
`2bd066d87f5bafd315be9f40889d0a60b9e58e0b`:
[preference field](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/ls/lsutil/userpreferences.go),
[initialization configuration](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/lsp/server.go), and
[checker hint production](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/ls/inlay_hints.go).
Native `GetTypeAtLocation` and its alias-aware type printer supply structured
label parts. Vize neither parses hover strings nor reconstructs generic types.

Preserve labels, kinds, padding, tooltips, commands and opaque data. Map positions
and writable edits through complete length-preserving authored spans. Generated
scaffolding, unsupported edit spans and out-of-range hints are declined. Optional
label locations use the existing canonical dependency projection; unmappable
private locations are omitted without replacing the native label. Prop and
translation decorations remain document-only. When native types are unavailable,
the server retains only proven Vue builtin source facts and declines unknown
reactive types. The existing document-only compatibility helper keeps its
retained explicit-generic controls; it is not the server's checker authority.

Whole structured projection controls cover UTF-16/CRLF, alias locations, writable
edits, commands, tooltip/padding/data retention and generated-span refusal. Actual
CLI stdio tests compare whole vectors with an independent stock native process on
the retained script bytes, including original/changed/repaired buffers, ranges,
plain JS/TS and generic aliases. Only known corresponding source-file identities
and physical dependency identities are normalized; label fields and ranges stay.
The sole stdout reader captures every decoded RPC envelope before enqueue,
including unconsumed notifications and parser/EOF errors; these are not raw wire
frames. Original inputs/configs and whole native/actual vectors are saved before
URI normalization. After both readers join, each process retains exit status and
raw stderr, plus the executable hash, version output and harness source identity.
Both native and Vize shutdown must succeed. These captures run on Actions.
The stock native oracle asserts the shutdown response, closes its input transport,
and requires actual exit success through the pinned server's accepted EOF route.
The retained standard native `exit` trace instead returned status 1 with
`context canceled`; that upstream limitation is neither accepted as success nor
claimed fixed. Vize still requires its standard `exit` with stdin held open.
The stock oracle initializes with the pinned required nullable `processId` and
declared pull-diagnostic capabilities. Before opening a document, it asserts and
acknowledges the complete pinned configuration-watch registration request, which
the native initialization handler awaits synchronously. Its retained bare script has two unused
binding suggestions for `label`/`items`, whose uses occur in the SFC template.
Assert that complete native diagnostic vector separately from the complete clean
SFC diagnostic vector. Authored plain JS/TS controls consume their computed value;
no suggestion/style preference is disabled and no diagnostic is filtered.
The authored stock JS config also publishes the complete TS5055 overwrite-input
error for `Oracle.js`, despite that document's empty pull-diagnostic vector. The
other three stock config publications are empty. Assert the whole publication
family after joined shutdown separately from every requested document vector,
including source/code/severity/message/range and config URI. Retain the original
authored options without adding `noEmit` or claiming config diagnostic parity.
The sole stdout reader preserves these envelopes before response matching; this
adds no native request, process, wait-order dependency or production change.
Checker-disabled and genuinely unavailable-runtime original RPCs are retained. Existing prop/i18n/resident
laws and the entire legacy differential corpus remain required.

The existing native reference/CSS guard is faithfully extracted before adding
mandatory hint qualification. It retains every original command/status guard;
no phase sampling, instruction ceiling or cold/warm timing budget changes.
The parent configured-session and private bulk qualifications must be retained
when replaying onto their actual merged main, with all source caps at 350.

Source review, exact-head native Actions, protected full suites, signed merge
with the verified reporter trailer and external release inclusion are pending.
This record makes no measured latency, Program-count or 10x speed claim.
Native File and the broader LSP/typechecker history gates remain unfinished.

The post-0.433 queue composition relocates only this fix's canonical link to an existing paragraph boundary; every incoming clause, original input, production body and full native oracle is retained. Fresh exact-head Actions and the current admitted-prefix composition are required before protected delivery.

The protected candidate `2ff25f358a001b0074c20fd132a136c6e5b975bc` exposed a source regression in the retained editor-only scorecard. Published v0.433.0 keeps authored inlay assistance enabled with `editor: true` and `typecheck: false`; the latter also disables the native backend. Preserve the complete original scorecard bytes and its positive Ref/ComputedRef assertions. Its captured complete request returned `null`, so removing those positives would accept the regression. The candidate was dequeued.

Record primitive builtin facts inside the existing Croquis declarator AST walk: only top-level constant identifiers calling value imports of Vue `ref`/`shallowRef` with one primitive literal, or a synchronous zero-parameter concise `computed` arrow whose numeric operands are literals or previously proven numeric builtin `.value` reads. Keep exact identifier spans and import ownership, decline generic/cast/type-only/non-Vue/shadowed/nested/unknown-call/bigint sources, and invalidate facts when parsing fails or a later binding replaces them. Store facts privately in the existing type owner without changing compatibility dumps or public constructible layouts. These facts preserve the existing compact source-owned wrappers; they are not native checker results.

Consume these facts in the already-required authored decoration analysis only when the native backend is disabled or cannot initialize. A live native query remains sole type authority, including a successful empty result. Preserve the existing request generation/cancellation guard, source buffer ownership, backend-off behavior, and all native structured-label/edit/range/JS/TS vectors. Add whole original scorecard/disabled/unavailable response vectors, numeric/import/unknown controls, exact UTF-16/CRLF ranges and edit invalidation. Fresh source/native Actions, protected full suites and release inclusion remain pending; no budget or legacy positive oracle is weakened.

The PR tooling planner explicitly defers the original Node Scorecard to the merge tier, so a green selected shard is not runtime evidence for that suite. Preserve all original positive assertions and add the independently captured published three-hint whole vector without changing its input. Run the complete original Node suite explicitly in the required inlay qualifier through the normal source-bound CLI build receipt, retaining raw sessions and the receipt on success or failure inside the existing uploaded subtree. Fresh exact-source execution and protected full suites remain required.

The protected candidate `888cedc` failed six unchanged Croquis instruction
ceilings (small full/compile/hoist on/off and large full/compile). The PR was
immediately removed from the queue and returned to Draft. The added primitive
fact hook had also run for ordinary compiler/full analysis; captured source
proves that unnecessary work, while precise numeric attribution remains bounded
by the retained candidate measurements. Request primitive facts only from the
editor's `BuiltinFacts` branch through the same Drawer AST pass. Default parsed,
parse-free and public statement paths specialize the hook away; private fact
storage stays absent until a proven editor fact exists. Keep all old inputs,
positive vectors, native authority and pinned ceilings unchanged. Fresh source,
native, original Scorecard and protected instruction qualification remain required.

The same protected candidate also failed the untouched granular CodeLens control:
its 573-byte FeatureIsolation.vue source requires the known string Ref and the
numeric computed value from `message.value.length * 2` with type checking off.
Retain that whole input and every positive assertion. Extend only the existing
AST primitive proof for a static nonoptional length read of a current-span-owned
primitive StringRef value; unknown objects, arrays, methods, casts and generics
stay untyped. [ECMAScript StringCreate](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-stringcreate)
sets its immutable length property to the Number conversion of the string's length.
Add the complete two-hint vector and refusal laws, and execute the original full
feature-isolation Node suite through the same source-bound build/receipt after
the original Scorecard command. Save each complete session family before
propagating either actual failure status. Native successful empty results keep
sole authority. No new output-vector acceptance or performance credit precedes
fresh exact-source Actions and protected qualification.

The exact `009ed0e` repair's existing instruction workflow retained one remaining
failure: `croquis_analyze_full_large` was 2,789,812 against the unchanged
2,786,473 ceiling. Authenticated original/current routine-return profiles
reconcile the 4,904 increase: script analysis +3,518 (ordinary statement
subtree +3,489), template analysis +1,392, and parent self work -6. Copy and
None-table drop costs increased only 14 and 3; they do not explain the total.
Restore the ordinary public statement dispatcher and variable loop byte-for-byte
from literal `f83e26d`. Only the demanded editor wrapper intercepts a top-level
variable declaration, retaining invalidation and per-declarator record-before-macro
ordering; it delegates every other statement to the original dispatcher. The
false parse specialization calls that original function directly. All old inputs,
whole controls, caps and native successful-empty authority remain intact. Fresh
source/native and original 100+4 instruction qualification are still required;
the static attribution and restoration give no new performance acceptance.

The later protected composition `ac88512` exceeded only the same full-large
ceiling: all three runs measured 2,786,595 against 2,786,473. Its actual parent
`b77f6a2` measured 2,785,602 in all three runs. Complete authenticated profiles
reconcile the 993 increase as script analysis -649 and template analysis +1,642;
the restored ordinary statement subtree decreased by 674. This is not a reason
to raise the ceiling or change any original hint or feature oracle.
Template reference checking currently traverses and hashes the same scope chain
once to resolve a name and again to mark a found binding as used. Resolve and
mark during one existing-order traversal, retaining the public mark-used API and
every builtin, shadow, parent, missing-name and undefined-reference rule. V-for
source reads also need only that one traversal. Whole state laws retain the old
two-traversal contract independently, including repeated reads, additional
parents and valid cycles. The required native helper selects both new laws;
all original whole editor/native controls and 100+4 ceilings remain unchanged.
Fresh source, native and instruction execution precedes renewed admission.
