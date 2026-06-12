# JSX/TSX Parity Inventory (Part of #1491)

A tracked matrix of the JSX/TSX parity surface for `vize_atelier_jsx`: which
reference cases (from `@vue/babel-plugin-jsx` and `vue-jsx-vapor`) are **covered**
by an executable test, the **backend** that covers them, and which are
**explicitly deferred** with a reason.

These tests assert **Vize's** output structure (helper calls, patch flags,
template strings, slot objects, `v-if`/`v-for`/`v-model` shapes), not byte-for-byte
babel parity — Vize emits through its own `vize_atelier_dom` / `vize_atelier_vapor`
codegen, so hoisting and block-tree details are intentionally Vize-shaped.

## Backend separation

Suites are split so a failure points at the correct backend:

| Suite        | File                    | Backend            | Passing | Ignored |
| ------------ | ----------------------- | ------------------ | ------- | ------- |
| VDOM parity  | `tests/parity_vdom.rs`  | `compile_to_dom`   | 41      | 2       |
| Vapor parity | `tests/parity_vapor.rs` | `compile_to_vapor` | 24      | 0       |
| TSX + modes  | `tests/parity_tsx.rs`   | both               | 10      | 0       |

The lower-level IR-shape tests (`tests/elements.rs`, `attributes.rs`,
`children.rs`, `components.rs`, `slots.rs`, `control_flow.rs`, `events.rs`,
`directives.rs`, `modes.rs`, `tsx.rs`, `dom.rs`, `vapor.rs`) remain the
fine-grained lowering coverage; this inventory tracks the **parity** layer.

## Covered categories

| Category         | Reference case                                                                       |     VDOM      | Vapor | TSX |
| ---------------- | ------------------------------------------------------------------------------------ | :-----------: | :---: | :-: |
| **Elements**     | intrinsic → `createElementBlock` / `_template`                                       |      ✅       |  ✅   |     |
|                  | component (PascalCase) resolution                                                    |      ✅       |  ✅   |     |
|                  | fragment (`<>…</>`)                                                                  |      ✅       |  ✅   |     |
|                  | member-expr / namespaced tags                                                        | ✅ (lowering) |       |     |
| **Attributes**   | static (inlined, no patch flag / baked into template)                                |      ✅       |  ✅   |     |
|                  | boolean (`disabled`)                                                                 |      ✅       |       |     |
|                  | dynamic `{expr}` → `PROPS` flag + dynamic-key array / `setProp`                      |      ✅       |  ✅   |     |
|                  | multiple dynamic keys collected                                                      |      ✅       |       |     |
|                  | spread `{...props}` → `FULL_PROPS` / `setDynamicProps`                               |      ✅       |  ✅   |     |
|                  | spread + static → `mergeProps`                                                       |      ✅       |       |     |
|                  | dynamic class → `normalizeClass` + `CLASS` flag / `setClass`                         |      ✅       |  ✅   |     |
|                  | array class binding                                                                  |      ✅       |       |     |
|                  | dynamic style → `normalizeStyle` + `STYLE` flag / `setStyle`                         |      ✅       |  ✅   |     |
|                  | namespaced `xlink:href`                                                              |      ✅       |       |     |
|                  | `key` (reserved, no patch flag)                                                      |      ✅       |       |     |
|                  | `ref` → `NEED_PATCH` flag                                                            |      ✅       |       |     |
| **Children**     | static text (no flag / baked)                                                        |      ✅       |  ✅   |     |
|                  | interpolation → `toDisplayString` + `TEXT` flag / `setText`                          |      ✅       |  ✅   |     |
|                  | mixed text + interpolation concatenation                                             |      ✅       |  ✅   |     |
|                  | free identifiers stay bare (no `_ctx.`)                                              |      ✅       |  ✅   |     |
|                  | member-expr interpolation stays bare                                                 |               |  ✅   |     |
| **Control flow** | `cond && <x/>` → `v-if` / `createIf`                                                 |      ✅       |  ✅   |     |
|                  | `cond ? <a/> : <b/>` → two-branch `v-if` / `createIf`                                |      ✅       |  ✅   |     |
|                  | `list.map(...)` → `v-for` (`UNKEYED_FRAGMENT`) / `createFor`                         |      ✅       |  ✅   |     |
|                  | `v-if` directive on element                                                          |      ✅       |       |     |
|                  | non-JSX `&&` stays interpolation (regression)                                        |      ✅       |       |     |
| **Directives**   | `v-model` on input → `vModelText` + `onUpdate:modelValue` / `applyTextModel`         |      ✅       |  ✅   |     |
|                  | `v-model` on checkbox → `vModelCheckbox`                                             |      ✅       |       |     |
|                  | `v-model` on component → `modelValue` prop                                           |      ✅       |  ✅   |     |
|                  | `v-model:foo` named arg → `foo` + `onUpdate:foo`                                     |      ✅       |       |     |
|                  | `v-show` → `vShow` / `applyVShow`                                                    |      ✅       |  ✅   |     |
|                  | `v-html` → `innerHTML` prop                                                          |      ✅       |       |     |
|                  | `v-text` → `textContent` prop                                                        |      ✅       |       |     |
|                  | custom directive `v-foo` → `resolveDirective`                                        |      ✅       |       |     |
| **Events**       | plain `onClick` → bind prop                                                          |      ✅       |  ✅   |     |
|                  | capture modifier (`onClickCapture`) → suffix key + `NEED_HYDRATION` / `_on{capture}` |      ✅       |  ✅   |     |
|                  | once modifier (`onClickOnce`)                                                        |      ✅       |       |     |
|                  | composed passive+capture                                                             |      ✅       |       |     |
| **Slots**        | object child named slots → `_withCtx` + `_: 1 /* STABLE */` / slot fn                |      ✅       |  ✅   |     |
|                  | render-prop child → default scoped slot                                              |      ✅       |       |     |
|                  | scoped named slot (destructured param stays bare)                                    |      ✅       |       |     |
|                  | plain element children → implicit default slot                                       |      ✅       |       |     |
| **TSX**          | typed arrow component → both backends                                                |               |       | ✅  |
|                  | generic component call `<List<number>/>`                                             |               |       | ✅  |
|                  | `as` cast inside interpolation (type-stripped by codegen)                            |               |       | ✅  |
|                  | non-null assertion in binding (type-stripped by codegen)                             |               |       | ✅  |
|                  | TS annotation rejected in `.jsx` mode                                                |               |       | ✅  |
| **Modes**        | default mode (vdom / vapor)                                                          |      ✅       |  ✅   | ✅  |
|                  | `"use vue:vapor"` / `"use vue:vdom"` prologue                                        |      ✅       |       | ✅  |
|                  | mixed module, per-component mode                                                     |               |       | ✅  |

## Explicitly deferred

### Compiler features Vize does not yet handle (ignored tests, never red)

Tracked as `#[ignore = "deferred: …"]` in `tests/parity_vdom.rs`:

| Case                                              | Reason                                                                                                                                                     | Tracking               |
| ------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- |
| `v-model` modifier-array form `{[val, ['trim']]}` | Lowers to a malformed nested `$event => ($event => …)` chain instead of attaching `.trim` as a model modifier.                                             | this inventory / #1491 |
| `v-model_lazy` suffix-modifier form               | babel-jsx's `v-model_lazy` / `v-model_number` underscore-suffix syntax resolves as a `model_lazy` **custom directive** instead of a lazy v-model modifier. | this inventory / #1491 |

### Type-level / resolve-type parity — deferred

`@vue/babel-plugin-jsx`'s `resolveType` (deriving runtime props/emits from TS
type annotations) and broader type-level parity require the type-resolution
infrastructure that is not yet wired through this crate. Deferred pending the
type-checker work in **#1497 / #1502**. The current TSX suite covers _syntax_
acceptance and type-stripping, not type-driven prop/emit generation.

### Ecosystem testbeds — deferred (network / CI infra)

Running the full `@vue/babel-plugin-jsx` + `vue-jsx-vapor` reference fixture
corpora and real-world component-library testbeds requires cloning external
repos and network access, which is unavailable in this build and gated on CI
infrastructure:

| Testbed  | Repo                | Reason deferred                           |
| -------- | ------------------- | ----------------------------------------- |
| Vuetify  | `vuetifyjs/vuetify` | Network clone + CI build matrix required. |
| Naive UI | `tusen-ai/naive-ui` | Network clone + CI build matrix required. |

These are tracked as the ecosystem-CI portion of **#1491** and are **not** closed
by this PR. This PR delivers the parity-suite + inventory **foundation** only.
