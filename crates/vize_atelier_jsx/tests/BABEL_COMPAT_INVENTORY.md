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
| `babel_compat/fixtures/corpus.json`       | the 100 inputs + the babel plugin options each is compiled with                                            |
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
| equivalent |   99 |
| divergent  |    0 |
| deferred   |    1 |

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
| `options/transform_on_on`           | wraps props in `_transformOn(...)`                  | `on` stays a prop by default                 | `BabelJsxOptions::transform_on` wraps via the helper | ✅      |
| `options/pragma`                    | emits `h("div", …)`, no `vue` import                | always emits Vue runtime helpers             | custom factory via additive pragma API               | ✅      |
| `options/merge_props_default`       | `mergeProps({class:"a"}, p, {class:c})`             | same                                         | no change                                            | ✅      |
| `options/merge_props_false`         | one object literal with a duplicate key             | always merges via `mergeProps`               | object spread + duplicate-key semantics (#3391)      | ✅      |
| `options/is_custom_element_default` | `<my-el/>` → `resolveComponent("my-el")`            | same                                         | no change                                            | ✅      |
| `options/is_custom_element_fn`      | matching tag becomes a string tag                   | predicate-selected tag uses element lowering | additive `isCustomElement` predicate API             | ✅      |
| `options/object_slots_default`      | `_isSlot(slots) ? slots : {default: () => [slots]}` | `toDisplayString(slots)` in the default slot | runtime slot-object check; calls evaluate once       | ✅      |
| `options/object_slots_false`        | `{default: () => [slots]}`                          | `toDisplayString(slots)` in the default slot | additive option preserves the raw default-slot child | ✅      |
| `options/resolve_type_off`          | JSX replaced, types untouched                       | equivalent render output                     | no change                                            | ✅      |
| `options/resolve_type_on`           | appends `{props: {...}, name: "A"}`                 | no type-driven inference                     | deferred: needs #1497 / #1502                        | ⏸       |

`isCustomElement` accepts captured predicates and composes with `pragma`,
`mergeProps`, and `transformOn` through `BabelJsxCustomizations`. As in Babel, a
lexical JavaScript binding wins over conversion to a string tag.

## Elements and tags

| Case                             | Babel                                                            | Vize today                                         | Compat mode                                   | Verdict |
| -------------------------------- | ---------------------------------------------------------------- | -------------------------------------------------- | --------------------------------------------- | ------- |
| `elements/intrinsic`             | `createVNode("div", …)`                                          | `createElementBlock("div")`                        | no change                                     | ✅      |
| `elements/component_pascal`      | `resolveComponent("B")`                                          | same                                               | no change                                     | ✅      |
| `elements/unknown_lowercase`     | `<foo/>` → `resolveComponent("foo")`                             | same in opt-in compat mode; native stays intrinsic | non-HTML/SVG lowercase classification (#3391) | ✅      |
| `elements/dashed_lowercase`      | `resolveComponent("my-el")`                                      | same                                               | no change                                     | ✅      |
| `elements/svg_tag`               | `createVNode("circle", …)`                                       | same                                               | no change                                     | ✅      |
| `elements/mathml_tag`            | `<mi/>` → `resolveComponent("mi")` (only HTML+SVG are intrinsic) | same in opt-in compat mode; native stays intrinsic | same fix as `unknown_lowercase` (#3391)       | ✅      |
| `elements/member_tag`            | `createVNode(a.b.c, …)`                                          | `resolveDynamicComponent(a.b.c)` (#3421)           | no change                                     | ✅      |
| `elements/namespaced_tag`        | rejects: `getTag: JSXNamespacedName is not supported`            | rejects it too, naming the namespace (#3421)       | no change                                     | ✅      |
| `elements/fragment`              | `createVNode(Fragment, null, […])`                               | `createElementBlock(Fragment, …, STABLE_FRAGMENT)` | no change                                     | ✅      |
| `elements/nested_fragment_child` | nested `Fragment` vnode                                          | children spliced into the parent (#3421)           | no change                                     | ✅      |

### Tag shapes: what the three ✅ above are equivalent _up to_

Recorded so the verdicts are not read as byte equality (#3421,
`src/lower/element.rs`, `src/lower/name.rs`):

- **Member tags.** `resolveDynamicComponent` returns a non-string argument
  unchanged, so `<a.b.c/>` mounts exactly what babel's `createVNode(a.b.c, …)`
  mounts. The two differ only when the member expression evaluates to a
  **string**: babel makes it an element tag, Vize looks it up as a registered
  component and falls back to the string. That is what `<component :is>` means
  in a Vue template, so the divergence is Vue's own semantics rather than a
  lowering gap.
- **Nested fragments.** A JSX fragment in child position carries no props and
  cannot be keyed, so Vize splices its children into the parent instead of
  emitting a wrapper. Same DOM, same patch order, one vnode level fewer than
  babel's nested `Fragment`.
- **Namespaced tags.** Babel rejects _every_ `<ns:tag/>`. Vize rejects every
  namespace except `svg:` and `math:`, the two that name a real element
  namespace; `<svg:circle/>` stays a verbatim tag (pinned by
  `elements.rs::known_namespaced_element_names_are_preserved`). This is the one
  place Vize is deliberately more permissive than babel, and a compat mode is
  not expected to narrow it.

## Props and attributes

| Case                             | Babel                                       | Vize today                              | Compat mode          | Verdict |
| -------------------------------- | ------------------------------------------- | --------------------------------------- | -------------------- | ------- |
| `props/static_attr`              | `{type: "email"}`                           | same                                    | no change            | ✅      |
| `props/boolean_attr`             | `<input disabled/>` → `disabled: true`      | `disabled: ""`                          | emits `true` (#3391) | ✅      |
| `props/dynamic_attr`             | `{placeholder: p}`                          | same + `PROPS` flag                     | no change            | ✅      |
| `props/dashed_attrs`             | `data-foo` / `aria-label` kept verbatim     | same                                    | no change            | ✅      |
| `props/xlink_camel`              | `xlinkHref` → `"xlink:href"`                | keeps `xlinkHref`                       | rewrites it (#3391)  | ✅      |
| `props/xlink_colon`              | `"xlink:href"`                              | same                                    | no change            | ✅      |
| `props/class_dynamic`            | `{class: c}` (runtime normalizes)           | `normalizeClass(c)` + `CLASS`           | no change            | ✅      |
| `props/class_static_and_dynamic` | `{class: ["a", c]}`                         | `normalizeClass(["a", c])`              | no change            | ✅      |
| `props/style_dynamic`            | `{style: s}`                                | `normalizeStyle(s)` + `STYLE`           | no change            | ✅      |
| `props/style_merge_with_spread`  | `mergeProps` in source order                | same                                    | no change            | ✅      |
| `props/spread_only`              | props are the spread expression itself      | `normalizeProps(guardReactiveProps(p))` | no change            | ✅      |
| `props/spread_then_static`       | `mergeProps(p, {id: "x"})`                  | same                                    | no change            | ✅      |
| `props/on_merge_with_spread`     | `mergeProps({onClick: a}, p, {onClick: b})` | same                                    | no change            | ✅      |
| `props/key`                      | `{key: k}`, no flag                         | same                                    | no change            | ✅      |
| `props/ref`                      | `{ref: r}`                                  | same + `NEED_PATCH`                     | no change            | ✅      |
| `props/ref_in_for`               | no `ref_for` emitted                        | no `ref_for` emitted                    | no change            | ✅      |
| `props/dollar_prefixed`          | `{$foo: 1}`                                 | same                                    | no change            | ✅      |

## Events

| Case                     | Babel                        | Vize today                     | Compat mode | Verdict |
| ------------------------ | ---------------------------- | ------------------------------ | ----------- | ------- |
| `events/plain`           | `{onClick: h}`               | same + `PROPS`                 | no change   | ✅      |
| `events/capture`         | `{onClickCapture: h}`        | same + `PROPS, NEED_HYDRATION` | no change   | ✅      |
| `events/once`            | `{onClickOnce: h}`           | same + `PROPS, NEED_HYDRATION` | no change   | ✅      |
| `events/capture_passive` | `{onClickCapturePassive: h}` | same + `PROPS, NEED_HYDRATION` | no change   | ✅      |

## Directives

| Case                                       | Babel                                                          | Vize today                                                      | Compat mode                                   | Verdict |
| ------------------------------------------ | -------------------------------------------------------------- | --------------------------------------------------------------- | --------------------------------------------- | ------- |
| `directives/v_model_input`                 | `withDirectives(…, [[vModelText, val]])`                       | same                                                            | no change                                     | ✅      |
| `directives/v_model_arg`                   | `[[vModelText, val, "foo"]]` + `onUpdate:foo`                  | rejected outside Babel VDOM compatibility mode                  | accepts and passes the arg in Babel VDOM mode | ✅      |
| `directives/v_model_modifier_array`        | `[[vModelText, val, void 0, {trim: true}]]`                    | same                                                            | no change                                     | ✅      |
| `directives/v_model_underscore`            | `{lazy: true}` modifiers                                       | same                                                            | no change                                     | ✅      |
| `directives/v_model_arg_underscore`        | `[[vModelText, val, "foo", {trim: true}]]`                     | rejected outside Babel VDOM compatibility mode                  | accepts arg and modifiers in Babel VDOM mode  | ✅      |
| `directives/v_model_component`             | `modelValue` + `onUpdate:modelValue` props                     | same                                                            | no change                                     | ✅      |
| `directives/v_model_component_arg_mods`    | `argument`, `argumentModifiers`, `onUpdate:argument`           | same props, different literal order                             | no change                                     | ✅      |
| `directives/v_model_component_dynamic_arg` | computed prop + `"onUpdate:" + bar`                            | rejected: dynamic arguments need computed prop lowering (#3466) | same computed props in Babel VDOM mode        | ✅      |
| `directives/v_models`                      | expands to one prop pair per entry                             | same, via one `model` directive per entry (#3418)               | no change                                     | ✅      |
| `directives/v_models_mods`                 | adds `<arg>Modifiers` per entry                                | same props, different literal order (#3418)                     | no change                                     | ✅      |
| `directives/v_show_element`                | `[[vShow, vis]]`                                               | same                                                            | no change                                     | ✅      |
| `directives/v_show_component`              | `[[vShow, vis]]` on the component vnode                        | same                                                            | no change                                     | ✅      |
| `directives/v_html`                        | `{innerHTML: h}`                                               | same                                                            | no change                                     | ✅      |
| `directives/v_html_with_children`          | keeps the children too                                         | same                                                            | no change                                     | ✅      |
| `directives/v_text`                        | `{textContent: t}` raw                                         | `textContent: toDisplayString(t)`                               | raw in compat mode                            | ✅      |
| `directives/v_custom_arg`                  | `[[resolveDirective("custom"), val, "arg"]]`                   | same                                                            | no change                                     | ✅      |
| `directives/v_custom_array`                | unpacks `[val, 'arg', ['a','b']]` into value / arg / modifiers | same (#3421)                                                    | no change                                     | ✅      |
| `directives/v_dashed_custom`               | `resolveDirective("unknown-thing")`                            | same                                                            | no change                                     | ✅      |

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

| Case                                 | Babel                                 | Vize today                                     | Compat mode | Verdict |
| ------------------------------------ | ------------------------------------- | ---------------------------------------------- | ----------- | ------- |
| `slots/object_children`              | object child becomes the slots object | `withCtx` slots + `_: 1`                       | no change   | ✅      |
| `slots/render_prop_child`            | `{default: () => 'foo'}`              | `default: () => [createTextVNode("foo")]`      | no change   | ✅      |
| `slots/scoped_param`                 | `default: s => …`                     | `default: withCtx((s) => […])`                 | no change   | ✅      |
| `slots/v_slots_with_children`        | `{default: () => […], ...slots}`      | same keys + `1024 /* DYNAMIC_SLOTS */` (#3467) | no change   | ✅      |
| `slots/v_slots_only`                 | slots object passed as children       | same + `1024 /* DYNAMIC_SLOTS */` (#3467)      | no change   | ✅      |
| `slots/v_slots_object_literal`       | object literal becomes the slots      | `withCtx` slots + `_: 1` (#3418)               | no change   | ✅      |
| `slots/v_slots_object_with_children` | `{default: () => […], bar: …}`        | same two slots, other literal order (#3418)    | no change   | ✅      |
| `slots/element_children_default`     | `{default: () => […]}`                | `withCtx` default slot + `_: 1`                | no change   | ✅      |
| `slots/dynamic_slot_name`            | `{[n]: () => …}`                      | dynamic slot key + patch flag                  | no change   | ✅      |

### Forwarding an opaque slots object (#3467)

`v-slots={slots}` carries a value the compiler cannot see inside, so there are no
entries to expand into slot templates. It lowers to a relief `slots` directive on
the component, which `vize_atelier_core`'s slot codegen emits as a spread. Both
babel shapes are reproduced exactly: the forwarded value **is** the children
argument when nothing else contributes slots (`createVNode(B, null, slots)`), and
otherwise closes the object literal (`{default: () => […], ...slots}`) so a
forwarded entry overrides an authored one of the same name.

The slot flag is the part that needed deciding, not the spread:

- **No `_` flag is emitted beside a spread** — matching babel, and load-bearing.
  Only the no-`_` path runs `normalizeObjectSlots`, which binds raw entries to
  the owning instance and passes already-`withCtx`-wrapped ones through
  untouched via `rawSlot._n`.
- **Not `_: 2 /* DYNAMIC */`.** `updateSlots` then does a bare
  `extend(slots, children)` with no normalization, so a forwarded entry that is
  not already wrapped would render without the right instance context.
- **Not `_: 1 /* STABLE */`.** The child would never re-render when the
  forwarded slots change.
- The vnode instead carries `1024 /* DYNAMIC_SLOTS */`, which is what forces
  that update. Babel gets the same forced update for free by emitting no patch
  flags at all: `shouldUpdateComponent` falls back to "any children means
  update" for unoptimized vnodes, and Vize's output is always optimized.

Like every other row, these three are asserted on the **emitted code** only:
the compiled-output mount harness described above still does not exist, so the
`equivalent` verdict here is review-checked in CI, not executed. The flag choice
was confirmed once out of band by mounting both emitted shapes against
`vue@3.5.35` under `happy-dom` — identical `innerHTML` after mount and after the
forwarded slots changed, with `_: 1` and the no-patch-flag variants both going
stale — but that check is not committed and does not run in CI (#3391).

`v-slots` is therefore a compiler built-in
(`vize_s0::BUILTIN_DIRECTIVES`), not a user directive: a component-level
`v-slots` in a `.vue` template now spreads too, rather than compiling to the
`resolveDirective("slots")` lookup #3418 removed. A user directive named `slots`
collides with it exactly as one named `show` or `model` would.

### `v-slots` spellings where native Vize is stricter than babel

Native mode rejects all the shapes below. Babel VDOM compatibility adopts only
corpus-pinned self-contained literals; the remaining exclusions stay explicit
(#3418, #3467, `src/lower/v_slots.rs`):

- `v-slots` with no value, a quoted value (`v-slots="str"`), or a container
  literal (`v-slots={[…]}`) remains rejected. Babel VDOM compatibility forwards
  self-contained expression literals (`v-slots={1}`, including static template
  literals) as component children; native mode retains the strict slots-object
  diagnostic. Interpolated templates remain rejected because forwarding their
  raw source could leak nested JSX.
- `v-slots={() => …}` — a lone function is the _default slot_, not a slots
  object: babel forwards it as children and Vue wraps it as `{default: fn}`.
  Spreading it would contribute nothing, so Vize names it and points at
  `v-slots={{ default: … }}` (or the render-prop child form, which it supports).
- `v-slots:arg={…}` — `v-slots` takes no argument; the slot names are the object's
  keys.
- `v-slots` on a plain element — babel drops it silently and emits `[]` children.
- more than one `v-slots` on the same element — babel keeps the last and drops the
  rest silently.
- an object literal that names `default` on an element that also has children.
  Babel emits the `default` key twice and lets JavaScript keep the later one,
  silently discarding the children; Vize reports "Extraneous children found when
  component already has an explicit default slot." from the shared transform.

## Children

| Case                       | Babel                         | Vize today                         | Compat mode                    | Verdict |
| -------------------------- | ----------------------------- | ---------------------------------- | ------------------------------ | ------- |
| `children/static_text`     | `[createTextVNode("x")]`      | `"x"` as the children argument     | no change                      | ✅      |
| `children/text_interp_mix` | three children                | one concatenated `TEXT` child      | no change                      | ✅      |
| `children/comment_only`    | children `null`               | no children                        | no change                      | ✅      |
| `children/empty_expr`      | children `null`               | no children                        | no change                      | ✅      |
| `children/spread_child`    | `[...items]`                  | `toDisplayString(items)`, reported | spread into the children array | ✅      |
| `children/logical_and`     | `[c && vnode]`                | `v-if` with a comment placeholder  | no change                      | ✅      |
| `children/ternary`         | `[c ? a : b]`                 | two-branch `v-if` with keys        | no change                      | ✅      |
| `children/map_list`        | raw `list.map(…)` array child | `renderList` + `KEYED_FRAGMENT`    | no change                      | ✅      |

## `optimize: true` (patch flags and dynamic prop keys)

Compared against Vize's default, which is already fully optimized.

| Case                             | Babel                                 | Vize today                            | Compat mode        | Verdict |
| -------------------------------- | ------------------------------------- | ------------------------------------- | ------------------ | ------- |
| `optimize/static`                | no flag                               | no flag                               | no change          | ✅      |
| `optimize/class_only`            | `2`                                   | `2 /* CLASS */`                       | no change          | ✅      |
| `optimize/style_only`            | `4`                                   | `4 /* STYLE */`                       | no change          | ✅      |
| `optimize/text_only`             | raw child, no flag                    | `toDisplayString(t)` + `1 /* TEXT */` | raw child, no flag | ✅      |
| `optimize/class_and_props`       | `10, ["id"]`                          | `11 /* TEXT, CLASS, PROPS */, ["id"]` | drops `TEXT`       | ✅      |
| `optimize/spread`                | `16`                                  | `16 /* FULL_PROPS */`                 | no change          | ✅      |
| `optimize/ref`                   | `512`                                 | `512 /* NEED_PATCH */`                | no change          | ✅      |
| `optimize/key`                   | no flag                               | no flag                               | no change          | ✅      |
| `optimize/event`                 | `8, ["onClick"]`                      | same                                  | no change          | ✅      |
| `optimize/component_props`       | `8, ["foo"]`                          | same                                  | no change          | ✅      |
| `optimize/v_model_input`         | `8, ["onUpdate:modelValue"]`          | same                                  | no change          | ✅      |
| `optimize/slots_stability`       | `_: 1`                                | `_: 1 /* STABLE */`                   | no change          | ✅      |
| `optimize/scoped_slot_stability` | `_: 1`                                | `_: 1 /* STABLE */`                   | no change          | ✅      |
| `optimize/v_slots_stability`     | slots object as children, no `_` flag | same, no `_` flag (#3467)             | no change          | ✅      |
| `optimize/fragment`              | no flag                               | `64 /* STABLE_FRAGMENT */`            | no change          | ✅      |
| `optimize/map_list`              | raw array child                       | `renderList` + `KEYED_FRAGMENT`       | no change          | ✅      |

## Error cases and permissive babel edges

The corpus pins rejection parity and babel-permissive edges. Compat mode must not
silently accept a babel error; permissive rows document opt-in VDOM behavior.

| Case                                      | Babel                                                           | Vize today                                                        | Compat mode             | Verdict |
| ----------------------------------------- | --------------------------------------------------------------- | ----------------------------------------------------------------- | ----------------------- | ------- |
| `errors/v_model_non_lval`                 | rejects a non-assignable `v-model` target                       | rejects it too, naming the offending expression (closed by #3420) | no change               | ✅      |
| `errors/v_model_no_value`                 | rejects: "You have to use JSX Expression inside your v-model"   | rejects: "v-model is missing expression."                         | no change               | ✅      |
| `errors/v_models_not_array`               | rejects a non-array `v-models` value                            | rejects it too, naming the expected entry shape (closed by #3418) | no change               | ✅      |
| `errors/v_models_entry_not_array`         | rejects: "You should pass a Two-dimensional Arrays to v-models" | rejects it too, naming the offending entry (closed by #3418)      | no change               | ✅      |
| `errors/v_slots_not_object`               | forwards the value as children                                  | rejects it, naming the offending value (#3418, #3467)             | forward safe primitives | ✅      |
| `errors/v_slots_static_template`          | forwards the static template as children                        | same in Babel VDOM compat; native still rejects                   | forward literal         | ✅      |
| `errors/v_slots_not_object_with_children` | `{ default: () => ["x"], ...1 }`                                | same spread semantics in Babel VDOM compat                        | no change               | ✅      |
