Fixes #13314

Vue refines a tag type in `onCloseTag` only outside `v-pre`, so the element that carries `v-pre` and every descendant stay plain elements. Vize's SSR already did this through the compiler's frozen-element custody. The ordinary parse, the native L2 lowering and Patina kept the old classification, so DOM and Vapor resolved `<div v-pre><Child/></div>` as a component.

**Behavior change:** under `v-pre` (opener included), a component tag, `<component>`, `<slot>` and a structural `<template>` now compile to plain elements in DOM and Vapor, as in Vue and in Vize's SSR. Inputs without such a tag under `v-pre` compile byte for byte as before.

## Why the `frozen_elements` gate existed

#8504 made SSR keep literal `v-pre` elements and Vue's literal directive names. Its first full run changed default DOM and Patina output: inherited directive names (`v-bind:x` → `v-bindx`), own modifiers, and "literal classification changed default Component/Slot ownership". The fix then restored the ordinary parser, Patina and every default lowering to the shipped contract and kept both behaviors behind the compiler-only `parse_with_frozen_elements` ([decision](https://github.com/ubugeeei-prod/vize/blob/main/docs/davinci/decisions/2026-10-10-ssr-nested-dynamic-arguments.md#default-compatibility-and-source-bound-lowering-correction)). So the gate kept an SSR fix from changing DOM. It was not a claim about Vue. This PR moves only the classification out of the gate. Directive names keep that split.

## Changes

- **Armature (legacy parse):** every parse classifies a `v-pre` opener and its subtree as `Element`. HTML tree construction still reads the type the tag has outside `v-pre`, so a literal component neither leaves a table nor changes `<p>` scope. Before this, SSR's custody parse fostered `<Row>` out of `<table v-pre><tr><Row></Row></tr></table>` and reported a recovery. Its lanes now agree and nothing is reported. The new `parse_with_frozen_element_spans` records the frozen spans without directive-head custody.
- **DOM, Vapor and the NAPI template `compile` (legacy lane):** all three parse with span custody and transform with `transform_with_frozen_elements`, so Core no longer promotes a literal element to a component. Vapor's lowering keeps a literal `<component>` out of its dynamic-component path. Hoisting keeps `is` on a literal `<component>`, as the native lane does. The legacy deletion guard's Vapor transform witness follows the call to `transform_with_frozen_elements(`. The route is unchanged.
- **Native L1 → L2:** component, slot outlet and element follow the `v-pre` scope in every entry, not only under frozen custody. The native and legacy DOM lanes agree byte for byte on every case below.
- **Patina and L1 tag facts:** frozen elements classify as `Element`, so the registered and native lint lanes stay in parity.
- Decision record: `docs/davinci/decisions/2026-11-18-13314-v-pre-subtree-elements.md`.

## Oracle (`@vue/compiler-sfc` 3.5.35, `@vue/compiler-vapor` 3.6.0-beta.10)

| Input | Vue DOM | Vize DOM before | Vize DOM after | SSR (Vue = Vize, unchanged) | Vapor after (Vue) |
| --- | --- | --- | --- | --- | --- |
| `<div v-pre><div><Child>{{ x }}</Child></div></div>` | `createElementVNode("Child", null, "{{ x }}")` | `createVNode(_component_Child, …)` | as Vue | `<div><div><Child>{{ x }}</Child></div></div>` | static template, no component (same) |
| `<Child v-pre>{{ x }}</Child>` | `createElementBlock("Child", null, "{{ x }}")` | `createBlock(_component_Child, …)` | as Vue | `<Child>{{ x }}</Child>` | static template (same) |
| `<div v-pre><component is="x"/></div>` | `createElementVNode("component")` | `resolveDynamicComponent("x")` | `createElementVNode("component", { is: "x" })` | `<component is="x"></component>` | `<component is="x">` (Vue drops `is`) |
| `<div v-pre><my-comp/></div>` | `createElementVNode("my-comp")` | `createVNode(_component_my_comp)` | as Vue | `<my-comp></my-comp>` | static template (same) |
| `<div v-pre><slot/></div>` | `createElementVNode("slot")` | `renderSlot(_ctx.$slots, "default")` | as Vue | `<slot></slot>` | `<slot>` markup only (Vue also emits `createSlot`) |
| `<div v-pre><template v-if="a">x</template></div>` | `createElementVNode("template", { "v-if": "a" }, [createTextVNode("x")])` | `createElementVNode("template", { "v-if": "a" }, "x")` | unchanged | `<template v-if="a">x</template>` | static template (same) |
| `<div><Child/></div>` (control) | component | component | component (unchanged) | `ssrRenderComponent` | `createComponentWithFallback` (same) |

With `compileScript({ inlineTemplate: true })`, the inline render closures are byte-identical to Vue's for rows 1, 2, 4, 5 and 7. Rows 3 and 6 differ only in the two spellings noted above. Vize's SSR first `_push` line equals Vue's for rows 1–6.

## Lint changes (aligned with Vue)

- `a11y/no-autofocus` and `a11y/no-access-key` now report on a literal component tag under `v-pre` (`<Foo v-pre autofocus accesskey />`), since Vue renders a plain element there.
- `vue/no-textarea-mustache` no longer reports `{{x}}` in `<slot v-pre><textarea>{{x}}</textarea></slot>`. The native L2 lowering used to lower a `<slot v-pre>` as an outlet whose fallback was not frozen, so `{{x}}` became an interpolation.

## Updated pins

- `vize_armature` `frozen.rs`: the ordinary parse now keeps `<Child>` under `v-pre` literal (it was pinned as `Component`).
- `vize_l1_to_l2` `frozen.rs` (2 tests): the ordinary lowering of `<slot v-pre>` is an element, like the explicit custody.
- `vize_l1_to_l2` `emit_outlets.rs`: `<slot v-pre>{{ not }} an interpolation</slot>` now renders Vue's literal element and text (the pin rendered `_toDisplayString(not)` in a slot fallback).
- `vize_atelier_dom` `v_pre_freezes_its_subtree.rs`: the recorded "known gap" (`<MyComp v-pre>` resolved as a component) is closed and pinned to Vue's output.
- `vize_l1` and `vize_patina` tag-kind pins for `v-pre` subtrees, plus the lint pins listed above.
- `vize_atelier_ssr` `literal_slot_boundaries`: the authentic-ca default DOM/Vapor packets stay immutable. `public-default-13314.corrections.json` corrects only the `code` of the six literal slot rows. Both normal-outlet rows are unchanged.
- `tests/tooling/compiler-css-api-custody.test.mjs`: the registered SSR case count goes from 103 to 104 for the new differential case, as earlier case additions did.
- `tools/support/compat/davinci/legacy-deletion-guard.ts`: the Vapor compatibility-transform witness now names `transform_with_frozen_elements(`. The route itself still exists.

## Tests

- Rust: DOM (both lanes, exact bodies), SSR (both lanes, exact `_push` lines), Vapor (whole modules), SFC (whole DOM/SSR/Vapor modules plus inline closures against Vue), L2 dump, parser classification and tree construction.
- Compiler differential case `compiler/sfc/v-pre-subtree-elements-13314` (SSR module bytes, CLI-generated).
- Local failure sets for the touched crates, default and `legacy` features, match `origin/main` by test name. The remaining failures need Node runtime packages or a real checker.
- compile-bench: RUN_ID

🤖 Generated with [Claude Code](https://claude.com/claude-code)
