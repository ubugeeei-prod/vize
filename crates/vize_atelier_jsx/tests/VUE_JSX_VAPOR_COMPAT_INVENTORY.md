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

| Case                                | Verdict    | Mounted behavior                                                                |
| ----------------------------------- | ---------- | ------------------------------------------------------------------------------- |
| `children/dynamic-text`             | equivalent | initial DOM and three text updates retain the authored element                  |
| `props/dynamic-attributes`          | equivalent | attributes and text update while the authored element remains live              |
| `props/spread-updates`              | equivalent | spread attributes update repeatedly on the same live element                    |
| `props/spread-before-static`        | equivalent | ordered source merges, replacements and restoration retain the authored element |
| `props/spread-after-static`         | equivalent | ordered source merges, replacements and restoration retain the authored element |
| `props/ordered-spread-chain`        | equivalent | ordered source merges, replacements and restoration retain the authored element |
| `props/spread-class-style`          | equivalent | ordered source merges, replacements and restoration retain the authored element |
| `events/spread-listener-order`      | equivalent | ordered source merges, replacements and restoration retain the authored element |
| `props/ordered-spread-reads`        | equivalent | every source evaluates once in authored order on each reactive publication      |
| `props/dynamic-value`               | equivalent | falsy, array and nested-VNode attribute values match both JSX references        |
| `props/dynamic-boolean-value`       | equivalent | value-sensitive boolean property updates match both real JSX references         |
| `props/dynamic-aria-value`          | equivalent | value-sensitive ARIA attribute updates match both real JSX references           |
| `props/dynamic-xlink-value`         | equivalent | value-sensitive namespaced attributes match both real JSX references            |
| `props/dynamic-type-value`          | equivalent | value-sensitive input type attributes match both real JSX references            |
| `events/click-closure`              | equivalent | two real clicks run the captured closure and retain the button                  |
| `children/logical-and`              | divergent  | only vue-jsx-vapor replaces the branch across truthy value changes              |
| `control-flow/conditional-branch`   | equivalent | falsy, array and nested-VNode conditions match both real JSX references         |
| `props/live-input-value`            | equivalent | the live input value and sibling output update without replacing either         |
| `directives/v-model-text`           | equivalent | input events and direct state patches preserve the live model binding           |
| `directives/v-show-value`           | equivalent | value-sensitive visibility updates match both real JSX references               |
| `directives/v-show-component-value` | equivalent | value-sensitive component visibility matches both real JSX references           |
| `props/dynamic-class-value`         | equivalent | value-sensitive class values match each backend's own JSX reference             |
| `props/dynamic-style-value`         | equivalent | value-sensitive style values match each backend's own JSX reference             |
| `components/class-value`            | equivalent | value-sensitive component class fallthrough matches both real JSX references    |
| `directives/v-html-value`           | equivalent | value-sensitive raw HTML matches each backend's own JSX reference               |
| `children/dynamic-value`            | equivalent | value-sensitive child values match each backend's own JSX reference             |
| `components/scoped-default-slot`    | equivalent | component props and a scoped slot closure update independently                  |
| `components/named-slot-updates`     | equivalent | named slot captures and component props update independently                    |
| `elements/fragment-updates`         | equivalent | fragment children update in place without a wrapper element                     |
| `slots/object-children`             | equivalent | object-child default and named slots update independently                       |
| `directives/v-model-component`      | equivalent | component `modelValue` round-trips through events and patches                   |
| `directives/v-model-component-arg`  | equivalent | an explicit model argument binds the named prop and event                       |
| `directives/v-model-checkbox`       | equivalent | checkbox `checked` follows change events and patches                            |
| `directives/v-model-radio`          | equivalent | radio selection follows change events and patches                               |
| `directives/v-model-select`         | equivalent | select state patches update the selection                                       |
| `directives/v-model-modifier-array` | equivalent | the modifier-array form trims input                                             |
| `directives/v-model-trim`           | equivalent | the underscore `trim` modifier trims input                                      |
| `directives/v-model-lazy`           | equivalent | the `lazy` modifier updates on change only                                      |
| `directives/v-model-number`         | equivalent | the `number` modifier stores numbers                                            |
| `directives/v-text`                 | equivalent | text content follows each update                                                |
| `directives/v-if-chain`             | equivalent | `v-if` / `v-else-if` / `v-else` switch and update branches                      |
| `directives/v-for-keyed`            | equivalent | keyed `v-for` reorders, removes and appends with reuse                          |
| `directives/v-slot-default`         | equivalent | a `v-slot` default scoped slot updates independently                            |
| `directives/v-slot-named`           | equivalent | a named `<template v-slot:…>` updates independently                             |
| `directives/custom`                 | equivalent | an in-scope custom directive receives a reactive getter                         |
| `directives/v-pre`                  | equivalent | `v-pre` is inert on JSX expressions                                             |
| `directives/v-cloak`                | equivalent | `v-cloak` renders no attribute                                                  |
| `props/key-replacement`             | equivalent | a key change replaces the element; content updates keep it                      |
| `tsx/typed-expressions`             | equivalent | TSX assertions erase without changing updates                                   |
| `control-flow/keyed-map-list`       | divergent  | only vue-jsx-vapor recreates keyed `.map` items                                 |
| `events/capture-suffix`             | divergent  | only vue-jsx-vapor ignores the `Capture` name suffix                            |
| `events/once-suffix`                | divergent  | only vue-jsx-vapor ignores the `Once` name suffix                               |
| `components/dynamic-component`      | divergent  | only Vize resolves `<component is>` dynamically                                 |
| `events/capture-modifier`           | divergent  | Vize ignores the underscore `capture` modifier                                  |
| `events/once-modifier`              | divergent  | Vize ignores the underscore `once` modifier                                     |
| `events/stop-modifier`              | divergent  | Vize ignores the underscore `stop` modifier                                     |
| `events/prevent-modifier`           | divergent  | Vize ignores the underscore `prevent` modifier                                  |
| `directives/custom-modifiers`       | divergent  | Vize passes no custom directive modifiers                                       |
| `directives/v-once`                 | divergent  | Vize keeps updating a `v-once` subtree                                          |
| `props/ref-callback`                | divergent  | Vize throws on a function `ref`                                                 |

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

## Dialect surface

Each row lists the mounted cases that exercise one surface; the oracle fails
when a case serves no surface or a row names an unknown case.

| Surface                                  | Cases                                                                                                                                                               |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Intrinsic elements and static attributes | `children/dynamic-text`, `props/dynamic-attributes`                                                                                                                 |
| Fragments                                | `elements/fragment-updates`                                                                                                                                         |
| Components, props and fallthrough        | `components/scoped-default-slot`, `components/named-slot-updates`, `components/class-value`, `directives/v-show-component-value`                                    |
| Dynamic components                       | `components/dynamic-component`                                                                                                                                      |
| Dynamic attribute values                 | `props/dynamic-value`, `props/dynamic-boolean-value`, `props/dynamic-aria-value`, `props/dynamic-xlink-value`, `props/dynamic-type-value`, `props/live-input-value` |
| Class and style                          | `props/dynamic-class-value`, `props/dynamic-style-value`, `props/spread-class-style`                                                                                |
| Spread props                             | `props/spread-updates`, `props/spread-before-static`, `props/spread-after-static`, `props/ordered-spread-chain`, `props/ordered-spread-reads`                       |
| Keys and refs                            | `props/key-replacement`, `props/ref-callback`                                                                                                                       |
| Text, interpolation and child values     | `children/dynamic-text`, `children/dynamic-value`                                                                                                                   |
| Conditional expressions                  | `children/logical-and`, `control-flow/conditional-branch`                                                                                                           |
| List expressions                         | `control-flow/keyed-map-list`                                                                                                                                       |
| Conditional directives                   | `directives/v-if-chain`                                                                                                                                             |
| List directive                           | `directives/v-for-keyed`                                                                                                                                            |
| Visibility                               | `directives/v-show-value`, `directives/v-show-component-value`                                                                                                      |
| Raw HTML and text content                | `directives/v-html-value`, `directives/v-text`                                                                                                                      |
| Element models                           | `directives/v-model-text`, `directives/v-model-checkbox`, `directives/v-model-radio`, `directives/v-model-select`                                                   |
| Model modifiers                          | `directives/v-model-modifier-array`, `directives/v-model-trim`, `directives/v-model-lazy`, `directives/v-model-number`                                              |
| Component models                         | `directives/v-model-component`, `directives/v-model-component-arg`                                                                                                  |
| Slots                                    | `slots/object-children`, `directives/v-slot-default`, `directives/v-slot-named`, `components/scoped-default-slot`, `components/named-slot-updates`                  |
| Render-once and inert directives         | `directives/v-once`, `directives/v-pre`, `directives/v-cloak`                                                                                                       |
| Custom directives                        | `directives/custom`, `directives/custom-modifiers`                                                                                                                  |
| Listeners                                | `events/click-closure`, `events/spread-listener-order`                                                                                                              |
| Listener name suffixes                   | `events/capture-suffix`, `events/once-suffix`                                                                                                                       |
| Listener underscore modifiers            | `events/capture-modifier`, `events/once-modifier`, `events/stop-modifier`, `events/prevent-modifier`                                                                |
| TSX                                      | `tsx/typed-expressions`                                                                                                                                             |

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
