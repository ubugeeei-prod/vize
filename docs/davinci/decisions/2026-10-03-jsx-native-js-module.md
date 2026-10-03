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

The seven committed sources in
`davinci/vize_l4/tests/fixtures/jsx-js-module-vue-3.5.35.json` are lowered by
the genuine parser → owning L2 File → L3 decisions → L4 module path. Rust
compares the complete native code and complete serialized map to that fixture.
The Node tooling suite independently transforms every original source using
`@babel/core` 7.29.0 and `@vue/babel-plugin-jsx` 2.0.1, with the recorded explicit
options. It compares the complete independent module and map, then executes
both complete module bodies against the actual Vue 3.5.35 runtime.

The suite compares pre-mount VNode roles and real `createRenderer` host trees
for fourteen executions, including re-rendered genuine parameter reads,
numeric/boolean/null values, escaped Unicode reads, both helper collisions,
multiple roots, a real lexical component and an original imported component.
Only the test loader's runtime/import addresses are adapted. The original
imported-component binding and named export remain in the native module.
A deliberately wrong numeric scalar fails the runtime comparison.

Native formatting and native maps are independent of Babel's formatting and
maps. The evidence grants runtime parity for these seven complete sources;
it does not claim upstream code/map byte parity or full JSX compatibility.

## Remaining work

- TSX profiles are refused even when their body otherwise fits. Unsupported
  TS syntax already refuses the lower completed owner. Type erasure is not
  implemented or advertised.
- Default function exports are still refused by the lower File producer;
  named function exports and original imports are covered here.
- Module directives/hashbangs, member tags, unknown intrinsic names,
  component slot children, JSX entity/whitespace normalization and duplicate
  static attributes have typed whole-target refusals.
- Compound/unary expressions, module initialization, dynamic attributes,
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
