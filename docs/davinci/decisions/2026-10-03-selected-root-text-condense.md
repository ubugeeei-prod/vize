# Selected original Vue 3 root text condensation

Paired issues: [#6835](https://github.com/ubugeeei-prod/vize/issues/6835)
for the source provider, and [#6838](https://github.com/ubugeeei-prod/vize/issues/6838)
for its genuinely dependent canonical consumer.

`NativeTemplateComponent::prepare_condensed_root_text` derives a sealed
`NativeRootText` only from the authentic descriptor-selected ordinary Vue 3
owner and its actual original direct root `NativeChild`. The receipt keeps
the genuine SourceBlock, template index, intrinsic JS/TS grammar, immutable
arena child address, original ordinal, complete raw text and absolute source
span. Its sole profile is default Vue 3 condense with retained comments.
Same-byte reparses, copied sources, sibling slots and bare Components cannot
mint or substitute this origin. A short current-root-cursor join compares the
actual immutable root slot in O(1); wrapper moves and pending-vector growth do
not substitute a wrapper address for original backing identity.

The algorithm follows the original [Vue 3.5.35 parser](https://github.com/vuejs/core/blob/v3.5.35/packages/compiler-core/src/parser.ts#L795):
TAB/LF/FF/CR/SPACE only. An all-ASCII-whitespace first/last root event is omitted;
comment/comment, comment/element and element/comment gaps are omitted; an
element/element gap is omitted only when its original content contains LF or
CR. Other blank gaps become one ASCII space. Mixed text collapses ASCII runs
without trimming edges. NBSP, U+0085, other Unicode whitespace and BOM survive.
The original tree and all raw spans remain unchanged, including omitted events.

Identity, omission and one-space results allocate no arena bytes. Only changed
mixed text uses the existing L0 arena StringBuilder, with no heap staging
string, normal legacy dependency, second parse, extra tree walk or pipeline
stage. Adjacent kinds are derived from the actual root slice, never a caller
neighborhood or a precomputed second traversal.

Foreign Components, nested children, non-text events, reported parser recovery
or unsupported observations, and raw ampersand/entity text have distinct typed
refusals. Text entities await a genuine existing once-only decoder join.
Nested pre/RCDATA/namespace ancestry is not inferred from a root receipt.
Unreported missing-close frames remain original structural observations and
cannot confer whole-component/File completion; this bounded text receipt does
not claim such completion. The dependent L2 receiver must retain its original
whole-body guards, prefix facts and sticky interruption/refusal rules.

The provider retains 37 complete independent official parser AST/source cases:
28 bounded eligible roots and nine explicit deferred entity/nested/pre/SVG/
recovery controls. The complete original root-slot packet includes every
ordinal and source byte, even omitted text. Native laws compare every eligible
root event/window and test nonzero Unicode source custody, original allocation
behavior, actual owner moves, foreign reparses, nested scopes, entity refusals
and original unsupported parser observations. All 38 independent pinned
compiler oracle tests pass locally. Hosted exact-head native/strict checks
and protected Stack/queue source suites, unchanged all-100 measurement and
actual merge remain acceptance requirements.

The next owned dependent L2 slice will add `root_text` to the existing original
root walk, returning `Option<NodeId>`: omitted source events advance the actual
cursor without a fabricated node or empty text op. Existing `child` and nested
body refusals remain intact. Complete source-built module/runtime/map equality
belongs to that consumer change, alongside normal completion and interruption
laws. Native nested whitespace, text entities, inherited pre/RCDATA, all Vue
profiles/dialects, runtime families and complete product routes remain unfinished.

The existing [explicit native SFC DOM product](./2026-10-03-native-sfc-scriptless-dom.md)
has an intentional preserve-whitespace family. This additive provider does not
change that product, its fixtures or default route. Product fix-history gates
remain open; no complete Vue 3 or product admission is granted by root text.
