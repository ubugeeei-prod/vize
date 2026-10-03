# Native JavaScript JSX module emission

Tracked in #6838, #6839, #6840 and #6829. This is the next dependent JSX
source slice after native Stack #7504's owning File, L3 view and scalar payload.
It does not close those roadmap issues or replace a legacy product route.

## Decision

`vize_l4::jsx::emit_js_module` accepts the actual retained
`NativeJsxAnalysis`, never a source string, forged role list or independent
AST/File pair. `decision_for` checks the original owner before indexing its
existing flat decision rows. Equal numeric positions from a second parse have
no authority. Owner movement, a foreign-owner lookup and a genuine embedded
source-window refusal have regression coverage.

For an admitted whole JavaScript module, L4 replaces only the original JSX
root spans and copies all intervening authored bytes. Module imports, named
exports, functions, line endings and non-JSX comments remain original. Scalar
containers retain the actual role-valued L3 payload and genuine resolved File
reads; their authored spelling and interior comments remain intact. Empty JSX
comment containers preserve the comments without creating children.

HTML/SVG intrinsic tags, simple actual component bindings without slot children,
the selected static attributes, nested static elements, single-line entity-free
text and scalar children emit calls to the checked Vue DOM vocabulary. Both
`createVNode` and required `createTextVNode` imports use aliases absent from
every actual File declaration and reference across all lexical scopes. A
nested parameter cannot capture a generated helper, and an unresolved original
read cannot acquire an accidental new import binding.

The existing append-only `Writer` emits the complete module. Its `Recorded`
and `NoLinks` paths have identical code and helper sets. Native Source Map v3
output carries the complete original source, per-line copied-source anchors,
JSX ranges and named original reads/components, with UTF-16 coordinates. The
synthetic runtime import is unmapped. No serialization, reparse, legacy normal
dependency, extra level stage or parallel metadata table is introduced.

## Independent execution evidence

The eleven committed sources in
`davinci/vize_l4/tests/fixtures/jsx-js-module-vue-3.5.35.json` are lowered by
the genuine parser → owning L2 File → L3 decisions → L4 module path. Rust
compares the complete native code and complete serialized map to that fixture.
The Node tooling suite independently transforms every original source using
`@babel/core` 7.29.0 and `@vue/babel-plugin-jsx` 2.0.1, with the recorded explicit
options. It compares the complete independent module and map, then executes
both complete module bodies against the actual Vue 3.5.35 runtime.

The suite compares pre-mount VNode roles and real `createRenderer` host trees
for eighteen executions, including re-rendered genuine parameter reads,
numeric/boolean/null values, escaped Unicode reads, both helper collisions,
multiple roots, a real lexical component and an original imported component.
Only the test loader's runtime/import addresses are adapted. The original
imported-component binding and named export remain in the native module.
A deliberately wrong numeric scalar fails the runtime comparison.

Native formatting and native maps are independent of Babel's formatting and
maps. The evidence grants runtime parity for these eleven complete sources;
it does not claim upstream code/map byte parity or full JSX compatibility.

## Review corrections

The actual source Actions run 37114370403 found the helper's forbidden
`format!` allocation and a weak comment containment assertion. The helper now
uses the existing compact-string macro. The test reparses only the emitted
module as a dev oracle and compares its entire ordered original-comment list;
no lint exception, fixture allowlist or assertion policy was weakened.

Independent pinned-transform execution found two target-policy gaps. An actual
parser comment with a factory pragma selects the authored `h` function, and
`search` is classified as a registered component by this pinned Vue plugin.
Neither behavior is implemented in the native target, so both now refuse the
whole module with original-source spans. Opening- and closing-tag comments also refuse
until their emission placement is implemented; empty-container and non-JSX
comments remain covered.

Four genuine whole-module fixtures add CR, CRLF, U+2028 and U+2029. The existing
shared map resolver recognizes these JavaScript line boundaries in its same
monotonic generated scan and existing source line-start table. Copied-source
links split at those same boundaries only for the recording sink. All seven
previous LF fixture code and map values remain exactly unchanged. Independent
VLQ decoding verifies the named `message` read at source line 2 and generated
line 3 for every new fixture, alongside actual Vue execution. Factory and
registered-component refusal references execute complete pinned modules and
compare full host trees.

## Remaining work

- TSX profiles are refused even when their body otherwise fits. Unsupported
  TS syntax already refuses the lower completed owner. Type erasure is not
  implemented or advertised.
- Default function exports are still refused by the lower File producer;
  named function exports and original imports are covered here.
- Module directives/hashbangs, member tags, unknown intrinsic names (including pinned `search`),
  component slot children, JSX entity/whitespace normalization and duplicate
  static attributes, factory pragmas and tag comments have typed whole-target refusals.
- Compound/unary expressions, unrelated ordinary scalar initializers and non-JSX
  return expressions, dynamic attributes,
  directives, spreads and fragments remain outside the completed lower view.
- Full JSX/TSX targets, dialect matrices, instruction-count queue validation,
  and compiler/product routing remain unfinished. Product legacy retirement
  still requires the existing fix-history gates.

## Validation and integration

Focused Rust module/ownership/refusal laws and the independent pinned runtime
tooling laws pass locally. Source acceptance belongs to exact-head Actions;
the protected queue must validate the full suites and instruction budgets.
The dependent PR joins the verified GitHub native Stack. Queued or green
source checks alone are not actual merge completion.

The current public union follows literal main `8501146f`, conserves the
reviewed compiler/fixture payloads and incoming source decisions, and uses
the official combined inventories. An unexpected restoration of older bottom
heads was inspected before publication; no unreviewed head is overwritten.
Native Stack relinking uses complete existing PR URLs and compares remote
OIDs before/after; guarded atomic publication uses those inspected expected
heads. Fresh source Actions, each full/instruction queue candidate and actual
merge remain separate required gates. Local compilation and installs remain
stopped under the team disk constraint.
