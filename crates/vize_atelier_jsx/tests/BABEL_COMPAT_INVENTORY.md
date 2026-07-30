# `@vue/babel-plugin-jsx` Compatibility Inventory (#3391)

The compatibility surface between Vize's JSX/TSX compiler and
`@vue/babel-plugin-jsx`, row by row, measured against the **real plugin** rather
than from memory.

This is the sibling of [`PARITY_INVENTORY.md`](./PARITY_INVENTORY.md). That file
tracks which reference cases have an executable test and asserts _Vize-shaped_
output. This file tracks whether Vize's **semantics** match babel's, and what an
opt-in compat mode (#3391) is expected to change.

## The oracle

| Artifact                                  | Role                                                                                                       |
| ----------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `babel_compat/fixtures/corpus.json`       | the 95 inputs + the babel plugin options each is compiled with                                             |
| `babel_compat/oracle.mjs`                 | runs the corpus through the real plugin; `--write` records, `--check` verifies                             |
| `babel_compat/fixtures/babel-output.json` | committed ground truth (generated — do not hand-edit)                                                      |
| `../babel_compat_oracle.rs`               | snapshots babel's output beside Vize's, per category, with a verdict per row                               |
| `babel_compat/verdicts.rs`                | the executable form of this file's verdict column                                                          |
| `tests/tooling/babel-jsx-oracle.test.ts`  | re-derives the recording from the installed plugin in CI; also asserts this file covers exactly the corpus |

**Which oracle each assertion uses.** The recording is produced by executing the
real `@vue/babel-plugin-jsx` (pinned `2.0.1` via the `babel-jsx-oracle` catalog),
so babel's side is never guessed. The Rust suite then compares **recorded babel
output against Vize output as a side-by-side snapshot with a declared verdict**,
not as an equality assertion — Vize is not trying to be byte-identical (see
_Global divergences_). The verdict column is where the semantic judgement lives,
and it is asserted to cover the corpus exactly in both directions.

**The `equivalent` verdicts are review-checked, not executed.** Upgrading them to
executed means mounting both outputs against a real Vue runtime and comparing
rendered DOM plus patch behavior on update. The repo has no harness that does
this today: the closest things are `playground/e2e/vrt/vapor-runtime.spec.ts`
(a Playwright smoke over the playground app, not over compiled output) and
`npm/fresco/src/testing/mount.ts` (an in-process `@vue/runtime-core` mount for
Fresco's own renderer). The cheapest viable path is the latter's shape rather
than an importmap + browser harness: `happy-dom` and `vue` are already workspace
catalog entries, so each corpus case's two emitted modules can be evaluated and
mounted in-process, then diffed on `innerHTML` after mount and after a state
update. Tracked on #3391; deliberately not claimed here.

Regenerate / verify:

```text
node crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs --check
cargo test -p vize_atelier_jsx --test babel_compat_oracle
node --test tests/tooling/babel-jsx-oracle.test.ts
```

## Totals

Pinned by `babel_compat_verdict_totals` in `../babel_compat_oracle.rs`, so these
numbers cannot drift from the verdict table.

| Verdict    | Rows |
| ---------- | ---: |
| equivalent |   68 |
| divergent  |   25 |
| deferred   |    2 |

## Global divergences

These hold for nearly every row and are deliberately **not** repeated in the
table; the per-row verdicts speak only about render semantics.

1. **Module shape.** Babel rewrites the JSX expression in place, so
   `const A = () => <div/>` stays a `const A = …`. Vize emits a standalone
   `export function render(_ctx, _cache)` for arrow-body components (block-body
   components are rebuilt as `defineComponent`). Drop-in compat requires
   preserving the surrounding module.
2. **Block tree.** Babel emits bare `createVNode` and never calls `openBlock`.
   Vize emits `openBlock()` + `createElementBlock` / `createBlock`.
3. **Patch flags are inverted.** Babel's **default** is `optimize: false` — no
   patch flags at all — and its own README warns that `optimize: true` "may skip
   certain re-renders". Vize is always fully optimized. Vize's default output
   already matches babel `optimize: true` almost exactly (see the `optimize`
   rows), so the gap is the _unoptimized_ direction: compat mode needs to be able
   to emit patch-flag-free output.
4. **Control flow is not lowered by babel.** `cond && <x/>`, `c ? <a/> : <b/>`
   and `list.map(…)` stay plain JavaScript. Vize lowers them to `v-if` / `v-for`
   (`createCommentVNode`, `renderList`, keyed fragments). Same rendered DOM and
   same reordering behavior, so those rows are `equivalent`, but the shape differs.

## Compat-mode meaning under `vapor`

`@vue/babel-plugin-jsx` is a vdom-era plugin: every output shape it defines is a
`createVNode` tree. Compat mode under `jsxMode: "vapor"` therefore has no defined
meaning and is **unsupported and diagnosed**, not silently ignored. This is a
deliberate answer, recorded here so it is not relitigated.

## Options surface

| Case                                | Babel                                               | Vize today                                   | Compat mode                                          | Verdict |
| ----------------------------------- | --------------------------------------------------- | -------------------------------------------- | ---------------------------------------------------- | ------- |
| `options/transform_on_off`          | `on` object passed through as a prop                | same                                         | no change                                            | ✅      |
| `options/transform_on_on`           | wraps props in `_transformOn(...)`                  | no such option; `on` stays a prop            | add `transformOn`                                    | ❌      |
| `options/pragma`                    | emits `h("div", …)`, no `vue` import                | always emits Vue runtime helpers             | add `pragma`                                         | ❌      |
| `options/merge_props_default`       | `mergeProps({class:"a"}, p, {class:c})`             | same                                         | no change                                            | ✅      |
| `options/merge_props_false`         | one object literal with a duplicate key             | always merges via `mergeProps`               | add `mergeProps: false`                              | ❌      |
| `options/is_custom_element_default` | `<my-el/>` → `resolveComponent("my-el")`            | same                                         | no change                                            | ✅      |
| `options/is_custom_element_fn`      | matching tag becomes a string tag                   | always resolved as a component               | add `isCustomElement`                                | ❌      |
| `options/object_slots_default`      | `_isSlot(slots) ? slots : {default: () => [slots]}` | `toDisplayString(slots)` in the default slot | treat a lone expression child as a slot object       | ❌      |
| `options/object_slots_false`        | `{default: () => [slots]}`                          | `toDisplayString(slots)` in the default slot | raw child, plus an `enableObjectSlots: false` option | ❌      |
| `options/resolve_type_off`          | JSX replaced, types untouched                       | equivalent render output                     | no change                                            | ✅      |
| `options/resolve_type_on`           | appends `{props: {...}, name: "A"}`                 | no type-driven inference                     | deferred: needs #1497 / #1502                        | ⏸       |

## Elements and tags

| Case                             | Babel                                                            | Vize today                                               | Compat mode                                  | Verdict |
| -------------------------------- | ---------------------------------------------------------------- | -------------------------------------------------------- | -------------------------------------------- | ------- |
| `elements/intrinsic`             | `createVNode("div", …)`                                          | `createElementBlock("div")`                              | no change                                    | ✅      |
| `elements/component_pascal`      | `resolveComponent("B")`                                          | same                                                     | no change                                    | ✅      |
| `elements/unknown_lowercase`     | `<foo/>` → `resolveComponent("foo")`                             | stays an intrinsic element                               | classify any non-HTML/SVG tag as a component | ❌      |
| `elements/dashed_lowercase`      | `resolveComponent("my-el")`                                      | same                                                     | no change                                    | ✅      |
| `elements/svg_tag`               | `createVNode("circle", …)`                                       | same                                                     | no change                                    | ✅      |
| `elements/mathml_tag`            | `<mi/>` → `resolveComponent("mi")` (only HTML+SVG are intrinsic) | stays an intrinsic element                               | same fix as `unknown_lowercase`              | ❌      |
| `elements/member_tag`            | `createVNode(a.b.c, …)`                                          | `resolveComponent("a.b.c")` — a name string              | emit the member expression                   | ❌      |
| `elements/namespaced_tag`        | rejects: `getTag: JSXNamespacedName is not supported`            | silently emits tag `a:b`                                 | reject with a diagnostic                     | ❌      |
| `elements/fragment`              | `createVNode(Fragment, null, […])`                               | `createElementBlock(Fragment, …, STABLE_FRAGMENT)`       | no change                                    | ✅      |
| `elements/nested_fragment_child` | nested `Fragment` vnode                                          | `resolveComponent("Fragment")` — unresolvable at runtime | use the `Fragment` symbol                    | ❌      |

## Props and attributes

| Case                             | Babel                                       | Vize today                              | Compat mode            | Verdict |
| -------------------------------- | ------------------------------------------- | --------------------------------------- | ---------------------- | ------- |
| `props/static_attr`              | `{type: "email"}`                           | same                                    | no change              | ✅      |
| `props/boolean_attr`             | `<input disabled/>` → `disabled: true`      | `disabled: ""`                          | emit `true`            | ❌      |
| `props/dynamic_attr`             | `{placeholder: p}`                          | same + `PROPS` flag                     | no change              | ✅      |
| `props/dashed_attrs`             | `data-foo` / `aria-label` kept verbatim     | same                                    | no change              | ✅      |
| `props/xlink_camel`              | `xlinkHref` → `"xlink:href"`                | keeps `xlinkHref`                       | rewrite the camel form | ❌      |
| `props/xlink_colon`              | `"xlink:href"`                              | same                                    | no change              | ✅      |
| `props/class_dynamic`            | `{class: c}` (runtime normalizes)           | `normalizeClass(c)` + `CLASS`           | no change              | ✅      |
| `props/class_static_and_dynamic` | `{class: ["a", c]}`                         | `normalizeClass(["a", c])`              | no change              | ✅      |
| `props/style_dynamic`            | `{style: s}`                                | `normalizeStyle(s)` + `STYLE`           | no change              | ✅      |
| `props/style_merge_with_spread`  | `mergeProps` in source order                | same                                    | no change              | ✅      |
| `props/spread_only`              | props are the spread expression itself      | `normalizeProps(guardReactiveProps(p))` | no change              | ✅      |
| `props/spread_then_static`       | `mergeProps(p, {id: "x"})`                  | same                                    | no change              | ✅      |
| `props/on_merge_with_spread`     | `mergeProps({onClick: a}, p, {onClick: b})` | same                                    | no change              | ✅      |
| `props/key`                      | `{key: k}`, no flag                         | same                                    | no change              | ✅      |
| `props/ref`                      | `{ref: r}`                                  | same + `NEED_PATCH`                     | no change              | ✅      |
| `props/ref_in_for`               | no `ref_for` emitted                        | no `ref_for` emitted                    | no change              | ✅      |
| `props/dollar_prefixed`          | `{$foo: 1}`                                 | same                                    | no change              | ✅      |

## Events

| Case                     | Babel                        | Vize today                     | Compat mode | Verdict |
| ------------------------ | ---------------------------- | ------------------------------ | ----------- | ------- |
| `events/plain`           | `{onClick: h}`               | same + `PROPS`                 | no change   | ✅      |
| `events/capture`         | `{onClickCapture: h}`        | same + `PROPS, NEED_HYDRATION` | no change   | ✅      |
| `events/once`            | `{onClickOnce: h}`           | same + `PROPS, NEED_HYDRATION` | no change   | ✅      |
| `events/capture_passive` | `{onClickCapturePassive: h}` | same + `PROPS, NEED_HYDRATION` | no change   | ✅      |

## Directives

| Case                                    | Babel                                                          | Vize today                                                      | Compat mode             | Verdict |
| --------------------------------------- | -------------------------------------------------------------- | --------------------------------------------------------------- | ----------------------- | ------- |
| `directives/v_model_input`              | `withDirectives(…, [[vModelText, val]])`                       | same                                                            | no change               | ✅      |
| `directives/v_model_arg`                | `[[vModelText, val, "foo"]]` + `onUpdate:foo`                  | rejected: "v-model argument is not supported on plain elements" | accept and pass the arg | ❌      |
| `directives/v_model_modifier_array`     | `[[vModelText, val, void 0, {trim: true}]]`                    | same                                                            | no change               | ✅      |
| `directives/v_model_underscore`         | `{lazy: true}` modifiers                                       | same                                                            | no change               | ✅      |
| `directives/v_model_arg_underscore`     | `[[vModelText, val, "foo", {trim: true}]]`                     | rejected as above                                               | accept and pass the arg | ❌      |
| `directives/v_model_component`          | `modelValue` + `onUpdate:modelValue` props                     | same                                                            | no change               | ✅      |
| `directives/v_model_component_arg_mods` | `argument`, `argumentModifiers`, `onUpdate:argument`           | same props, different literal order                             | no change               | ✅      |
| `directives/v_models`                   | expands to one prop pair per entry                             | same, via one `model` directive per entry (#3418)               | no change               | ✅      |
| `directives/v_models_mods`              | adds `<arg>Modifiers` per entry                                | same props, different literal order (#3418)                     | no change               | ✅      |
| `directives/v_show_element`             | `[[vShow, vis]]`                                               | same                                                            | no change               | ✅      |
| `directives/v_show_component`           | `[[vShow, vis]]` on the component vnode                        | same                                                            | no change               | ✅      |
| `directives/v_html`                     | `{innerHTML: h}`                                               | same                                                            | no change               | ✅      |
| `directives/v_html_with_children`       | keeps the children too                                         | same                                                            | no change               | ✅      |
| `directives/v_text`                     | `{textContent: t}` raw                                         | `textContent: toDisplayString(t)`                               | assign raw              | ❌      |
| `directives/v_custom_arg`               | `[[resolveDirective("custom"), val, "arg"]]`                   | same                                                            | no change               | ✅      |
| `directives/v_custom_array`             | unpacks `[val, 'arg', ['a','b']]` into value / arg / modifiers | passes the whole array as the value                             | unpack the array form   | ❌      |
| `directives/v_dashed_custom`            | `resolveDirective("unknown-thing")`                            | same                                                            | no change               | ✅      |

### `v-models` spellings Vize rejects and babel accepts

Not corpus rows, because a compat mode is not expected to adopt them; recorded
here so the choice is not implicit (#3418, `src/lower/v_models.rs`):

- `v-models:x={…}` — babel reads the JSX namespace as a default prop name but
  then ignores each entry's own argument, so `v-models:x={[[a], [b, "b"]]}` binds
  `x` and `modelValue` and never `b`. The spelling is undocumented and its babel
  behavior is inconsistent, so Vize rejects it and points at the entry form.
- `v-models_lazy={…}` — the `_`-suffixed modifier spelling. Same reasoning: the
  per-entry modifier array expresses it exactly.

Vize also lets `v-models` through on a dashed lowercase tag (`<my-el/>`), which
it classifies as an intrinsic element but the DOM backend still resolves with
`resolveComponent` — matching babel, which treats it as a custom component.

## Slots

| Case                             | Babel                                 | Vize today                                   | Compat mode                           | Verdict |
| -------------------------------- | ------------------------------------- | -------------------------------------------- | ------------------------------------- | ------- |
| `slots/object_children`          | object child becomes the slots object | `withCtx` slots + `_: 1`                     | no change                             | ✅      |
| `slots/render_prop_child`        | `{default: () => 'foo'}`              | empty default slot — the value is dropped    | keep the non-JSX body                 | ❌      |
| `slots/scoped_param`             | `default: s => …`                     | `default: withCtx((s) => […])`               | no change                             | ✅      |
| `slots/v_slots_with_children`    | `{default: () => […], ...slots}`      | falls through to `resolveDirective("slots")` | implement `v-slots`                   | ❌      |
| `slots/v_slots_only`             | slots object passed as children       | falls through to `resolveDirective("slots")` | implement `v-slots`                   | ❌      |
| `slots/element_children_default` | `{default: () => […]}`                | `withCtx` default slot + `_: 1`              | no change                             | ✅      |
| `slots/dynamic_slot_name`        | `{[n]: () => …}`                      | warns and drops the slot                     | deferred: needs dynamic-slot lowering | ⏸       |

## Children

| Case                       | Babel                         | Vize today                        | Compat mode                    | Verdict |
| -------------------------- | ----------------------------- | --------------------------------- | ------------------------------ | ------- |
| `children/static_text`     | `[createTextVNode("x")]`      | `"x"` as the children argument    | no change                      | ✅      |
| `children/text_interp_mix` | three children                | one concatenated `TEXT` child     | no change                      | ✅      |
| `children/comment_only`    | children `null`               | no children                       | no change                      | ✅      |
| `children/empty_expr`      | children `null`               | no children                       | no change                      | ✅      |
| `children/spread_child`    | `[...items]`                  | `toDisplayString(items)`          | spread into the children array | ❌      |
| `children/logical_and`     | `[c && vnode]`                | `v-if` with a comment placeholder | no change                      | ✅      |
| `children/ternary`         | `[c ? a : b]`                 | two-branch `v-if` with keys       | no change                      | ✅      |
| `children/map_list`        | raw `list.map(…)` array child | `renderList` + `KEYED_FRAGMENT`   | no change                      | ✅      |

## `optimize: true` (patch flags and dynamic prop keys)

Compared against Vize's default, which is already fully optimized.

| Case                             | Babel                        | Vize today                                   | Compat mode         | Verdict |
| -------------------------------- | ---------------------------- | -------------------------------------------- | ------------------- | ------- |
| `optimize/static`                | no flag                      | no flag                                      | no change           | ✅      |
| `optimize/class_only`            | `2`                          | `2 /* CLASS */`                              | no change           | ✅      |
| `optimize/style_only`            | `4`                          | `4 /* STYLE */`                              | no change           | ✅      |
| `optimize/text_only`             | raw child, no flag           | `toDisplayString(t)` + `1 /* TEXT */`        | emit the raw child  | ❌      |
| `optimize/class_and_props`       | `10, ["id"]`                 | `11 /* TEXT, CLASS, PROPS */, ["id"]`        | drop the TEXT child | ❌      |
| `optimize/spread`                | `16`                         | `16 /* FULL_PROPS */`                        | no change           | ✅      |
| `optimize/ref`                   | `512`                        | `512 /* NEED_PATCH */`                       | no change           | ✅      |
| `optimize/key`                   | no flag                      | no flag                                      | no change           | ✅      |
| `optimize/event`                 | `8, ["onClick"]`             | same                                         | no change           | ✅      |
| `optimize/component_props`       | `8, ["foo"]`                 | same                                         | no change           | ✅      |
| `optimize/v_model_input`         | `8, ["onUpdate:modelValue"]` | same                                         | no change           | ✅      |
| `optimize/slots_stability`       | `_: 1`                       | `_: 1 /* STABLE */`                          | no change           | ✅      |
| `optimize/scoped_slot_stability` | `_: 1`                       | `_: 1 /* STABLE */`                          | no change           | ✅      |
| `optimize/v_slots_stability`     | slots object as children     | falls through to `resolveDirective("slots")` | implement `v-slots` | ❌      |
| `optimize/fragment`              | no flag                      | `64 /* STABLE_FRAGMENT */`                   | no change           | ✅      |
| `optimize/map_list`              | raw array child              | `renderList` + `KEYED_FRAGMENT`              | no change           | ✅      |

## Inputs babel rejects

A compat mode must reject what babel rejects, with a diagnostic — never silently
accept it.

| Case                              | Babel                                                           | Vize today                                                        | Compat mode         | Verdict |
| --------------------------------- | --------------------------------------------------------------- | ----------------------------------------------------------------- | ------------------- | ------- |
| `errors/v_model_non_lval`         | rejects a non-assignable `v-model` target                       | rejects it too, naming the offending expression (closed by #3420) | no change           | ✅      |
| `errors/v_model_no_value`         | rejects: "You have to use JSX Expression inside your v-model"   | rejects: "v-model is missing expression."                         | no change           | ✅      |
| `errors/v_models_not_array`       | rejects a non-array `v-models` value                            | rejects it too, naming the expected entry shape (closed by #3418) | no change           | ✅      |
| `errors/v_models_entry_not_array` | rejects: "You should pass a Two-dimensional Arrays to v-models" | rejects it too, naming the offending entry (closed by #3418)      | no change           | ✅      |
| `errors/v_slots_not_object`       | forwards the value as children                                  | emits a custom `slots` directive                                  | forward as children | ❌      |
