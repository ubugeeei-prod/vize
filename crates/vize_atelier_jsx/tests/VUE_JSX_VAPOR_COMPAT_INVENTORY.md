# vue-jsx-vapor semantic compatibility inventory

This inventory records behavior-level compatibility between Vize's native JSX
Vapor backend and the real `vue-jsx-vapor` package. Both generated modules run
under the same pinned Vue Vapor runtime. Each row mounts both components in
Happy DOM, applies the corpus updates, and compares normalized rendered DOM,
live form state, diagnostics, and element identity.

Every surface of the dialect listed under [Dialect surface](#dialect-surface)
has at least one mounted row. References compile with `transformVueJsxVapor`'s
defaults: the opt-in `macros` option and the VDOM `interop` mode are outside
the dialect this oracle measures.

## Pinned oracle

| Dependency      | Version       |
| --------------- | ------------- |
| `vue-jsx-vapor` | `3.2.26`      |
| `vue`           | `3.6.0-rc.10` |

Versions live in the `jsx-vapor-oracle` pnpm catalog. A version bump requires
a reviewed update to the executable verdict inventory.

## Verdicts

| Case                                             | Verdict    | Mounted behavior                                                                |
| ------------------------------------------------ | ---------- | ------------------------------------------------------------------------------- |
| `children/dynamic-text`                          | equivalent | initial DOM and three text updates retain the authored element                  |
| `props/dynamic-attributes`                       | equivalent | attributes and text update while the authored element remains live              |
| `props/spread-updates`                           | equivalent | spread attributes update repeatedly on the same live element                    |
| `props/spread-before-static`                     | equivalent | ordered source merges, replacements and restoration retain the authored element |
| `props/spread-after-static`                      | equivalent | ordered source merges, replacements and restoration retain the authored element |
| `props/ordered-spread-chain`                     | equivalent | ordered source merges, replacements and restoration retain the authored element |
| `props/spread-class-style`                       | equivalent | ordered source merges, replacements and restoration retain the authored element |
| `events/spread-listener-order`                   | equivalent | ordered source merges, replacements and restoration retain the authored element |
| `props/ordered-spread-reads`                     | equivalent | every source evaluates once in authored order on each reactive publication      |
| `props/dynamic-value`                            | equivalent | falsy, array and nested-VNode attribute values match both JSX references        |
| `props/dynamic-boolean-value`                    | equivalent | value-sensitive boolean property updates match both real JSX references         |
| `props/dynamic-aria-value`                       | equivalent | value-sensitive ARIA attribute updates match both real JSX references           |
| `props/dynamic-xlink-value`                      | equivalent | value-sensitive namespaced attributes match both real JSX references            |
| `props/dynamic-type-value`                       | equivalent | value-sensitive input type attributes match both real JSX references            |
| `events/click-closure`                           | equivalent | two real clicks run the captured closure and retain the button                  |
| `children/logical-and`                           | divergent  | only vue-jsx-vapor replaces the branch across truthy value changes              |
| `control-flow/conditional-branch`                | equivalent | falsy, array and nested-VNode conditions match both real JSX references         |
| `props/live-input-value`                         | equivalent | the live input value and sibling output update without replacing either         |
| `directives/v-model-text`                        | equivalent | input events and direct state patches preserve the live model binding           |
| `directives/v-show-value`                        | equivalent | value-sensitive visibility updates match both real JSX references               |
| `directives/v-show-component-value`              | equivalent | value-sensitive component visibility matches both real JSX references           |
| `props/dynamic-class-value`                      | equivalent | value-sensitive class values match each backend's own JSX reference             |
| `props/dynamic-style-value`                      | equivalent | value-sensitive style values match each backend's own JSX reference             |
| `components/class-value`                         | equivalent | value-sensitive component class fallthrough matches both real JSX references    |
| `directives/v-html-value`                        | equivalent | value-sensitive raw HTML matches each backend's own JSX reference               |
| `children/dynamic-value`                         | equivalent | value-sensitive child values match each backend's own JSX reference             |
| `components/scoped-default-slot`                 | equivalent | component props and a scoped slot closure update independently                  |
| `components/named-slot-updates`                  | equivalent | named slot captures and component props update independently                    |
| `elements/fragment-updates`                      | equivalent | fragment children update in place without a wrapper element                     |
| `slots/object-children`                          | equivalent | object-child default and named slots update independently                       |
| `directives/v-model-component`                   | equivalent | component `modelValue` round-trips through events and patches                   |
| `directives/v-model-component-arg`               | equivalent | an explicit model argument binds the named prop and event                       |
| `directives/v-model-checkbox`                    | equivalent | checkbox `checked` follows change events and patches                            |
| `directives/v-model-radio`                       | equivalent | radio selection follows change events and patches                               |
| `directives/v-model-select`                      | equivalent | select state patches update the selection                                       |
| `directives/v-model-modifier-array`              | equivalent | the modifier-array form trims input                                             |
| `directives/v-model-trim`                        | equivalent | the underscore `trim` modifier trims input                                      |
| `directives/v-model-lazy`                        | equivalent | the `lazy` modifier updates on change only                                      |
| `directives/v-model-number`                      | equivalent | the `number` modifier stores numbers                                            |
| `directives/v-text`                              | equivalent | text content follows each update                                                |
| `directives/v-if-chain`                          | equivalent | `v-if` / `v-else-if` / `v-else` switch and update branches                      |
| `directives/v-for-keyed`                         | equivalent | keyed `v-for` reorders, removes and appends with reuse                          |
| `directives/v-slot-default`                      | equivalent | a `v-slot` default scoped slot updates independently                            |
| `directives/v-slot-named`                        | equivalent | a named `<template v-slot:…>` updates independently                             |
| `directives/custom`                              | equivalent | an in-scope custom directive receives a reactive getter                         |
| `directives/v-pre`                               | equivalent | `v-pre` is inert on JSX expressions                                             |
| `directives/v-cloak`                             | equivalent | `v-cloak` renders no attribute                                                  |
| `props/key-replacement`                          | equivalent | a key change replaces the element; content updates keep it                      |
| `tsx/typed-expressions`                          | equivalent | TSX assertions erase without changing updates                                   |
| `control-flow/keyed-map-list`                    | divergent  | only vue-jsx-vapor recreates keyed `.map` items                                 |
| `events/capture-suffix`                          | divergent  | only vue-jsx-vapor ignores the `Capture` name suffix                            |
| `events/once-suffix`                             | divergent  | only vue-jsx-vapor ignores the `Once` name suffix                               |
| `components/dynamic-component`                   | divergent  | only Vize resolves `<component is>` dynamically                                 |
| `events/capture-modifier`                        | divergent  | Vize ignores the underscore `capture` modifier                                  |
| `events/once-modifier`                           | divergent  | Vize ignores the underscore `once` modifier                                     |
| `events/stop-modifier`                           | divergent  | Vize ignores the underscore `stop` modifier                                     |
| `events/prevent-modifier`                        | divergent  | Vize ignores the underscore `prevent` modifier                                  |
| `directives/custom-modifiers`                    | divergent  | Vize passes no custom directive modifiers                                       |
| `directives/v-once`                              | divergent  | Vize keeps updating a `v-once` subtree                                          |
| `props/ref-callback`                             | divergent  | Vize throws on a function `ref`                                                 |
| `options/on-object-value`                        | equivalent | value-sensitive pair agrees                                                     |
| `options/transform-on-value`                     | equivalent | value-sensitive pair agrees                                                     |
| `options/merge-props-class-value`                | equivalent | value-sensitive pair agrees                                                     |
| `options/merge-props-false-value`                | divergent  | value-sensitive pair records a divergence                                       |
| `options/object-slots-value`                     | divergent  | value-sensitive pair records a divergence                                       |
| `options/object-slots-false-value`               | divergent  | value-sensitive pair records a divergence                                       |
| `tsx/child-value`                                | equivalent | value-sensitive pair agrees                                                     |
| `props/class-static-and-dynamic-value`           | equivalent | value-sensitive pair agrees                                                     |
| `props/style-merge-value`                        | equivalent | value-sensitive pair agrees                                                     |
| `props/spread-only-value`                        | divergent  | value-sensitive pair records a divergence                                       |
| `props/spread-then-static-value`                 | equivalent | value-sensitive pair agrees                                                     |
| `props/on-merge-value`                           | equivalent | value-sensitive pair agrees                                                     |
| `props/key-value`                                | equivalent | value-sensitive pair agrees                                                     |
| `props/ref-value`                                | divergent  | value-sensitive pair records a divergence                                       |
| `props/ref-in-for-value`                         | divergent  | value-sensitive pair records a divergence                                       |
| `events/plain-value`                             | equivalent | value-sensitive pair agrees                                                     |
| `events/capture-value`                           | divergent  | value-sensitive pair records a divergence                                       |
| `events/once-value`                              | divergent  | value-sensitive pair records a divergence                                       |
| `events/capture-passive-value`                   | divergent  | value-sensitive pair records a divergence                                       |
| `directives/v-model-input-value`                 | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-model-arg-value`                   | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-model-modifier-array-value`        | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-model-lazy-value`                  | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-model-arg-trim-value`              | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-model-component-value`             | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-model-component-arg-mods-value`    | divergent  | value-sensitive pair records a divergence                                       |
| `directives/v-model-component-dynamic-arg-value` | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-models-value`                      | divergent  | value-sensitive pair records a divergence                                       |
| `directives/v-models-mods-value`                 | divergent  | value-sensitive pair records a divergence                                       |
| `directives/v-html-children-value`               | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-text-value`                        | equivalent | value-sensitive pair agrees                                                     |
| `directives/custom-arg-value`                    | equivalent | value-sensitive pair agrees                                                     |
| `directives/custom-array-value`                  | equivalent | value-sensitive pair agrees                                                     |
| `directives/dashed-custom-value`                 | equivalent | value-sensitive pair agrees                                                     |
| `slots/object-children-value`                    | divergent  | value-sensitive pair records a divergence                                       |
| `slots/render-prop-value`                        | divergent  | value-sensitive pair records a divergence                                       |
| `slots/scoped-param-value`                       | divergent  | value-sensitive pair records a divergence                                       |
| `slots/v-slots-with-children-value`              | divergent  | value-sensitive pair records a divergence                                       |
| `slots/v-slots-only-value`                       | divergent  | value-sensitive pair records a divergence                                       |
| `slots/v-slots-object-literal-value`             | divergent  | value-sensitive pair records a divergence                                       |
| `slots/v-slots-object-with-children-value`       | divergent  | value-sensitive pair records a divergence                                       |
| `slots/element-children-value`                   | equivalent | value-sensitive pair agrees                                                     |
| `slots/dynamic-slot-name-value`                  | equivalent | value-sensitive pair agrees                                                     |
| `children/text-mix-value`                        | divergent  | value-sensitive pair records a divergence                                       |
| `children/spread-child-value`                    | equivalent | value-sensitive pair agrees                                                     |
| `children/map-list-value`                        | divergent  | value-sensitive pair records a divergence                                       |
| `optimize/class-only-value`                      | equivalent | value-sensitive pair agrees                                                     |
| `optimize/style-only-value`                      | equivalent | value-sensitive pair agrees                                                     |
| `optimize/class-and-props-value`                 | equivalent | value-sensitive pair agrees                                                     |
| `optimize/spread-value`                          | divergent  | value-sensitive pair records a divergence                                       |
| `optimize/ref-value`                             | divergent  | value-sensitive pair records a divergence                                       |
| `optimize/key-value`                             | equivalent | value-sensitive pair agrees                                                     |
| `optimize/event-value`                           | equivalent | value-sensitive pair agrees                                                     |
| `optimize/v-model-input-value`                   | equivalent | value-sensitive pair agrees                                                     |
| `optimize/slots-stability-value`                 | divergent  | value-sensitive pair records a divergence                                       |
| `optimize/scoped-slot-stability-value`           | divergent  | value-sensitive pair records a divergence                                       |
| `optimize/v-slots-stability-value`               | divergent  | value-sensitive pair records a divergence                                       |
| `optimize/map-list-value`                        | divergent  | value-sensitive pair records a divergence                                       |
| `directives/v-model-checkbox-value`              | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-model-radio-value`                 | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-model-select-value`                | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-model-trim-value`                  | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-model-number-value`                | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-if-value`                          | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-for-value`                         | equivalent | value-sensitive pair agrees                                                     |
| `directives/v-slot-value`                        | equivalent | value-sensitive pair agrees                                                     |

The JSON corpus and verdict table are checked bidirectionally, so adding or
removing a case without a verdict fails. A `divergent` verdict names its
tracking issue and exactly the trace fields that differ (`dom`, `text`,
`elementIdentities`, `warnings`, `events`); the oracle fails when any other
field differs, and `expectedReference<Field>` pins the reference side of each
differing field. The runtime comparison is the source
of each `equivalent` verdict; generated-code shape is not used as a substitute
for mounted behavior.

The spread row is the mounted counterpart of the Babel oracle's equivalent
`props/spread_only` row. It replaces the complete spread object three times,
updates every spread value and sibling text, and then restores the initial
class with a new identity attribute. Both Vize/reference pairs must agree on
the complete DOM and diagnostics while the authored element retains identity
through every phase.

The event row dispatches two bubbling, cancelable `click` events through the
mounted button. Its receipt pins dispatch success, `defaultPrevented`, the
closure's rendered counter side effect, diagnostics, and element identity for
every phase before checking complete unmount.

The model row assigns a real input value before dispatching each bubbling,
cancelable `input` event, pins the complete dispatch and live-input receipt,
and interleaves a direct state patch. Both directions must update the mounted
input and sibling output without replacing any authored element.

The component rows resolve a registered VDOM fixture through each Vapor
compiler's component fallback. One passes a reactive prop and a scoped default
slot; the other passes a named `header` slot through `v-slots`. Both update the
prop and captured slot state independently. The mounted article, prop output,
slot containers and authored slot elements must all retain identity while their
complete DOM and diagnostics remain equal.

The dynamic-type-value, dynamic-value, dynamic-boolean-value,
dynamic-aria-value, dynamic-xlink-value, v-show-value,
v-show-component-value, logical-and and
conditional rows are the value-sensitive counterparts of the Babel oracle's equivalent
`props/static_attr`, `props/dynamic_attr`, `props/boolean_attr`,
`props/dashed_attrs`, `props/xlink_colon`, `directives/v_show_element`,
`directives/v_show_component`,
`children/logical_and` and `children/ternary` rows. Their mounted update
sequences pass `0`, an empty string, `NaN`, `null`, `undefined`, `false`, `true`,
an array and a nested VNode through both Vize/reference pairs. The comparison
retains complete DOM, visible text, diagnostics and element identity separately
for the VDOM and Vapor runtimes. The dynamic-value and dynamic-aria-value rows
apply each value to the same live DOM attribute, while the
dynamic-boolean-value row exercises DOM boolean-property coercion instead of
using those values only as conditions. The boolean row pins `disabled` as
present for the empty string, `true`, the array and the nested VNode, and as
absent for `0`, `NaN`, `null`, `undefined` and `false`. The ARIA row retains
`"false"` and `"true"` as meaningful attribute strings while removing only the
nullish phases. The type row pins an input attribute, the xlink row pins a
namespaced attribute, and the two v-show rows pin truthiness-driven inline
display updates on both an intrinsic element and a resolved component root.
The logical-and references agree on DOM and text but differ in
identity: Babel retains the truthy branch, while `vue-jsx-vapor` replaces it
when the truthy condition value changes. The executable verdict pins that
identity difference while still requiring identical DOM, text and diagnostics
across all phases.

The dynamic-class-value, dynamic-style-value, class-value, v-html-value and
child-value rows are the value-sensitive counterparts of the Babel oracle's
equivalent `props/class_dynamic`, `props/style_dynamic`,
`optimize/component_props`, `directives/v_html` and `optimize/text_only` rows.
JSX `class` follows `createVNode`, which normalizes only truthy non-string
values. Babel therefore renders `0`, `NaN` and `false` as class text and drops
nullish classes, while a resolved component's root receives fallthrough through
`mergeProps`. At some phases the two references
disagree with each other, without Vize:

- class, phases 0, 1, 2, 5, 6, 7 and 9: Vapor's `setClass` normalizes every
  value, while Babel's `createVNode` passes falsy values through unchanged.
- style, phases 1 to 7: after the empty string, Vue's VDOM keeps an empty
  `style` attribute that Vapor's `setStyle` never writes.
- v-html, phases 3 and 4: after `NaN`, Vue's VDOM renderer keeps the stale
  markup for `null` and `undefined`. `patchElement` clears `innerHTML` only
  when the previous value is truthy. Vapor's `setHtml` clears it.
- child value, phase 8: `vue-jsx-vapor` stringifies the nested VNode to
  `[object Object]`, while Babel renders its DOM.

Each verdict records those phases as `referenceConflicts`. The oracle still
requires Vize Vapor to equal `vue-jsx-vapor` and Vize VDOM to equal Babel in
every phase. It allows the two Vize backends to differ only at the recorded
phases, and only while the references still differ there. `expectedVdomText`
and `expectedVdomElementIdentities` pin the VDOM side of each conflict. The
differences are upstream's; Vize does not paper over either runtime.

Every Babel row also compiles VDOM with source maps enabled. That build uses
the Relief codegen instead of the L2 emitter, and its mounted trace must equal
the L2 trace.

Event steps wait for the clock to advance before dispatching, because Vue's
VDOM invoker ignores an event stamped no later than the listener's attachment.
TSX rows keep erasable type syntax in both emitted modules; the oracle strips
it with Node's `stripTypeScriptTypes` before linking, as a bundler would.

## Value coverage

Rows with `valueMatrix` drive one value position through `0`, `""`, `NaN`,
`null`, `undefined`, `false`, `true`, an array and a nested VNode, then `0`
again. Listener rows click after each value. `fixtures/value-coverage.json`
records the matrix, and the oracle enforces it:

- Every Babel row with an equivalent verdict has a value-sensitive mounted row
  linked by `babelCorpusId`. The only exceptions are `literal` rows, whose JSX
  expressions are all literals or empty (checked against the source), and
  `compile-error` rows (`errors/*`).
- Every equivalent `vue-jsx-vapor` row is value-sensitive itself or names the
  value-sensitive row that drives its value position.

A linked Babel row compiles the reference with its own plugin options. The
VDOM build switches to Babel compatibility mode when those options change
semantics (`transformOn`, `mergeProps`, `enableObjectSlots`) or when the row
sets `vdomCompat: "babel"`, because the Babel inventory documents that only
compat mode matches there. `vaporRejected` pins a Vapor build that Vize
refuses with a diagnostic; that row compares only the VDOM/Babel pair.

A VDOM divergence from Babel is recorded the same way as a Vapor one:
`vdomDiffers` and `vdomIssue`, with `expectedBabel<Field>` pinning the Babel side.

## Dialect surface

Each row lists the mounted cases that exercise one surface; the oracle fails
when a case serves no surface or a row names an unknown case.

| Surface                                  | Cases                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Intrinsic elements and static attributes | `children/dynamic-text`, `props/dynamic-attributes`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| Fragments                                | `elements/fragment-updates`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| Components, props and fallthrough        | `components/scoped-default-slot`, `components/named-slot-updates`, `components/class-value`, `directives/v-show-component-value`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| Dynamic components                       | `components/dynamic-component`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| Dynamic attribute values                 | `props/dynamic-value`, `props/dynamic-boolean-value`, `props/dynamic-aria-value`, `props/dynamic-xlink-value`, `props/dynamic-type-value`, `props/live-input-value`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| Class and style                          | `props/dynamic-class-value`, `props/dynamic-style-value`, `props/spread-class-style`, `options/merge-props-class-value`, `options/merge-props-false-value`, `props/class-static-and-dynamic-value`, `props/style-merge-value`, `optimize/class-only-value`, `optimize/style-only-value`, `optimize/class-and-props-value`                                                                                                                                                                                                                                                                                                                                               |
| Spread props                             | `props/spread-updates`, `props/spread-before-static`, `props/spread-after-static`, `props/ordered-spread-chain`, `props/ordered-spread-reads`, `props/spread-only-value`, `props/spread-then-static-value`, `optimize/spread-value`                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| Keys and refs                            | `props/key-replacement`, `props/ref-callback`, `props/key-value`, `props/ref-value`, `props/ref-in-for-value`, `optimize/ref-value`, `optimize/key-value`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| Text, interpolation and child values     | `children/dynamic-text`, `children/dynamic-value`, `children/text-mix-value`, `children/spread-child-value`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| Conditional expressions                  | `children/logical-and`, `control-flow/conditional-branch`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| List expressions                         | `control-flow/keyed-map-list`, `children/map-list-value`, `optimize/map-list-value`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| Conditional directives                   | `directives/v-if-chain`, `directives/v-if-value`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| List directive                           | `directives/v-for-keyed`, `directives/v-for-value`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| Visibility                               | `directives/v-show-value`, `directives/v-show-component-value`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| Raw HTML and text content                | `directives/v-html-value`, `directives/v-text`, `directives/v-html-children-value`, `directives/v-text-value`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| Element models                           | `directives/v-model-text`, `directives/v-model-checkbox`, `directives/v-model-radio`, `directives/v-model-select`, `directives/v-model-input-value`, `optimize/v-model-input-value`, `directives/v-model-checkbox-value`, `directives/v-model-radio-value`, `directives/v-model-select-value`                                                                                                                                                                                                                                                                                                                                                                           |
| Model modifiers                          | `directives/v-model-modifier-array`, `directives/v-model-trim`, `directives/v-model-lazy`, `directives/v-model-number`, `directives/v-model-arg-value`, `directives/v-model-modifier-array-value`, `directives/v-model-lazy-value`, `directives/v-model-arg-trim-value`, `directives/v-model-trim-value`, `directives/v-model-number-value`                                                                                                                                                                                                                                                                                                                             |
| Component models                         | `directives/v-model-component`, `directives/v-model-component-arg`, `directives/v-model-component-value`, `directives/v-model-component-arg-mods-value`, `directives/v-model-component-dynamic-arg-value`, `directives/v-models-value`, `directives/v-models-mods-value`                                                                                                                                                                                                                                                                                                                                                                                                |
| Slots                                    | `slots/object-children`, `directives/v-slot-default`, `directives/v-slot-named`, `components/scoped-default-slot`, `components/named-slot-updates`, `options/object-slots-value`, `options/object-slots-false-value`, `slots/object-children-value`, `slots/render-prop-value`, `slots/scoped-param-value`, `slots/v-slots-with-children-value`, `slots/v-slots-only-value`, `slots/v-slots-object-literal-value`, `slots/v-slots-object-with-children-value`, `slots/element-children-value`, `slots/dynamic-slot-name-value`, `optimize/slots-stability-value`, `optimize/scoped-slot-stability-value`, `optimize/v-slots-stability-value`, `directives/v-slot-value` |
| Render-once and inert directives         | `directives/v-once`, `directives/v-pre`, `directives/v-cloak`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| Custom directives                        | `directives/custom`, `directives/custom-modifiers`, `directives/custom-arg-value`, `directives/custom-array-value`, `directives/dashed-custom-value`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| Listeners                                | `events/click-closure`, `events/spread-listener-order`, `options/on-object-value`, `options/transform-on-value`, `props/on-merge-value`, `events/plain-value`, `events/capture-value`, `events/once-value`, `events/capture-passive-value`, `optimize/event-value`                                                                                                                                                                                                                                                                                                                                                                                                      |
| Listener name suffixes                   | `events/capture-suffix`, `events/once-suffix`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| Listener underscore modifiers            | `events/capture-modifier`, `events/once-modifier`, `events/stop-modifier`, `events/prevent-modifier`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| TSX                                      | `tsx/typed-expressions`, `tsx/child-value`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |

Run the focused oracle with:

```text
cargo test -p vize_atelier_jsx --test vue_jsx_vapor_compat_oracle
```

The ordered-source rows cover both static/object orders, two spread objects
separated by a reactive named prop, key removal and restoration, class/style
composition, and real listener side-effect order. Complete authored DOM traces
are pinned independently of the reference comparison. The getter row also
records each source read and the number of runtime prop publications: every
reactive effect reads the three sources once in source order and publishes one
combined prop state. The same shared lowering contract is checked through
retained template, native template, and VDOM mounted witnesses (#10458).
