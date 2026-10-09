# Native-element Vue component selectors

Issue: [#8328](https://github.com/ubugeeei-prod/vize/issues/8328).
Reporter: Danila Poyarkov (`dannote`).
Implementation base: actual main `3b36ed12c87a528115d3ddc301d7309ddfcda16b`.

The reported native element `is="vue:my-thing"` must select the component
`my-thing`, pass the other attributes and bindings as component props, and
keep the children as its default slot. Public L2 consumers must receive a
`ComponentOp`, rather than having to reconstruct compiler-specific semantics
from a native element. Plain `is="my-thing"` and bound `:is="view"` on native
elements retain their native attribute semantics.

## Reference and ownership

The [Vue attribute documentation](https://vuejs.org/api/built-in-special-attributes.html#is),
[Vue 3.5.43 parser](https://github.com/vuejs/core/blob/v3.5.43/packages/compiler-core/src/parser.ts),
and [component transform](https://github.com/vuejs/core/blob/v3.5.43/packages/compiler-core/src/transforms/transformElement.ts)
are the primary reference. Static attribute values decode before prefix
classification. `vue:` is case-sensitive; its remainder is the component
name, including an empty remainder. Explicit `<component is="...">` retains
its established dynamic-component behavior. Custom-element admission and
`v-pre` prevent native elements from being cast.

The retained parser classifies the component after its attributes are known.
The existing core transform consumes the selector and changes only semantic
identity, before the ordinary DOM/SSR/Vapor resolution, prop and slot paths.
It retains the original element and attribute locations. Closing tags are
matched against the authored tag before semantic identity changes.

The independent L1-to-L2 lowering collects the selector ordinal during its
existing attribute analysis, uses L1's attribute decoder on that selected
value, builds a `ComponentOp` with the borrowed decoded remainder, and
records the consumed selector as `drop.vue-is`. Other authored attributes,
bindings, children and element spans retain their existing ownership. The
native construction header uses its already prepared and decoded static
attribute directly; it neither decodes the value again nor calls a legacy
compiler. The original lossless carrier remains unchanged.

Table admission recognizes component-cast rows and cells before minting an
implicit `tbody` or `tr`. This prevents `<tr is="vue:my-row">` from acquiring
a wrapper that an equivalent named component does not have. Ordinary and
plain-`is` rows retain the existing HTML tree normalization. The check runs
only for candidate table row/cell tags. Encoded selectors use the same L1
decoder. The cold encoded table check and later owner lowering each prepare
the selected value; ordinary values borrow without allocating. There is no
second general attribute walk or entity implementation. No new pipeline stage,
serialization, dependency, public result field or persistent cache is added.

Reserved names `component`, `Component`, `slot` and `template` still name
static components after a cast. Existing spans and the original source
borrow distinguish them from authored dynamic tags, outlets and fragment
wrappers. A shared bounded L0 predicate reads only an opening tag's name and
delimiter, allocates nothing, and runs only on reserved-name branches.
Original bare `<component />` contracts remain unchanged. The SSR L4 and
Vapor L3 routes admit the consumed-selector provenance; private Vapor
admission receives the existing source borrow and assigns its existing
component-kind field before attachment, without an origin DTO or reparse.
Native construction lets the first static `is` decide, using its already
prepared attribute value. Candidate table admission checks `v-pre` with the
existing directive classifier in the same candidate-only attribute scan;
this also covers the public compatibility parser's carrier whose verbatim
auxiliary bit is unset. Both attribute orders retain the native `tbody`. Child whitespace and text
mode also follow the authored tag: casting a `pre` preserves its text, and
casting a `div` to a component named `pre` or `textarea` does not introduce
HTML preformatted or RCDATA behavior.

An ordinary authored `template` with a static Vue selector is also a
component. Its existing analyzed branch/for/slot facts keep structural
templates as fragments or slot carriers. Native construction waits until
its already prepared directive list is complete before consuming that
selector, so both attribute orders behave identically. Only the ordinary
cast bypasses the native plain-template refusal. Slot outlets, structural
templates and the bounded native script/style RAWTEXT refusal retain their
existing contracts; this change does not introduce raw-text entity decoding.

## Regression evidence

The [corpus fixture](../../../tests/_fixtures/differential/compiler/vue-is-component-8328/README.md)
preserves the original template and finite props/slot, encoded-value and
plain-native controls.

`davinci/vize_l1_to_l2/tests/vue_is_component.rs` exercises actual public L2
and native construction. It checks the original component/default slot,
ordered props and dynamic binding, authored spans and consumed-selector
provenance, once-decoded component names, native/plain/bound/case-sensitive
and verbatim controls, the empty remainder, and exact native-construction
whitespace and lossless carrier fidelity.

`crates/vize_atelier_vapor/tests/vue_is_component.rs` checks complete DOM,
SSR and Vapor code/preamble/maps/templates against equivalent named
components for eight authored pairs: the original report, props/default
slot, encoded prefix/value, ordinary template, once-decoded name, if/else, for, and table row.
Exact whole SSR code also pins plain native, bound native, explicit
custom-element and verbatim controls.

Before production changes, the five original L2 laws on the actual main
`6a75086cf305ea576117cc48ec68d1ad6a6f2cdd` base produced four genuine failures and one passing native control. The
recorded command was:

```sh
cargo test -p vize_l1_to_l2 --test vue_is_component -- --nocapture
```

The source fix then passed all ten L2/native and eight cross-backend laws
locally on Rust 1.99. Four reserved-name whole-function DOM/SSR oracles derive
from executed primary Vue 3.5.41 compilers and preserve Vize's existing
function formatting; the 3.5.43 resolver/parser sources were verified, but
that package could not be downloaded locally. A Vapor unit law independently
proves native admission and regular component IR for five finite casts,
including the original-tag delimiter boundary. The same changed targets and production libraries
passed Clippy with warnings denied. The existing assertion lint passed
without allowlist changes. Source Actions found one missing reviewed storage
row for the identity helper. Its exact observation is one L0 String import
and one bound use: the empty `drop.vue-is` after value allocates nothing.
The per-file factual inventory now records those counts without changing a
budget, cap or allowlist. Consumer import inventory is regenerated with
the existing scanner.

The full source Rust suite then caught existing JSX value tags whose
synthesized `is` binding keeps the original opening-name span. Reserved
component checks now reuse that span; a real cast's attribute span cannot
match it. The existing source-less graph tests retain their tag contract,
and KeepAlive admits both regular and dynamically selected children as
before. Default lint parsing retains its authored classification: all
three compilers install the existing native-tag policy, and the Patina
facade reads its already-retained authored surface tag. The complete
facade battery, 20 original native ARIA laws, 134 static/frozen JSX laws,
three cache laws and the original 18 cast laws pass. Local JSX runtime
execution requires an installed Vite native binding; Actions owns that
proof. These focused local results do not substitute for
fresh exact-head Actions, protected merge-queue checks or release proof.

Fresh source Actions passed all 17,098 Rust tests, including both original
JSX runtime tests, and all source tooling, package and canonical corpus
jobs. Protected candidate `ab7eb1e2229c028e986feceb37ecbc9080f788af`
then measured `patina_jsx_markup_one_root` at 45,990 instructions against
the unchanged 45,385 ceiling; the other 99 stage ceilings passed. The
candidate was withdrawn. Patina now chooses its existing surface/op tag
directly and borrows it once inside the already-selected L2 kind arm,
avoiding repeated facade dispatch. Authored tag and source-less special
template behavior stay identical, with no new field, copy or allocation.
The first successor's standalone measurement reached 45,650 instructions,
340 fewer but still 265 above the ceiling. The same existing tag/kind
selectors now expose their borrowed branches to callers with inline hints;
their reviewed bodies and immediate slot return stay identical. A fresh
measurement must establish the result of those hints.
The source matrix then found one stale factual L1 reference count after
the accessor rewrite removed an explicit type reference. The existing
scanner refreshes only Patina's element inventory row from five sites to
four. Production code, inventory rules, budgets and oracles stay unchanged.
The existing instruction workflow can measure the successor before queue
admission; it does not replace fresh protected queue qualification.

## Delivery

The fix is an independent PR with a conventional title and `Refs #8328`.
The single fixing commit must end with:

```text
Co-authored-by: Danila Poyarkov <dev@dannote.net>
```

This identity is bound to the original issue author and their public GitHub
profile. Preserve the final trailer through squash and verify it on actual
main. Do not close the report merely because a PR merges. Root owns the
finite protected merge admission and the official release, including
released artifact contents, the available version and the issue update.
Publication is unfinished until those checks succeed.
