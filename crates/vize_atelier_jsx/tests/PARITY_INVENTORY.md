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
| VDOM parity  | `tests/parity_vdom.rs`  | `compile_to_vdom`  | 41      | 2       |
| Vapor parity | `tests/parity_vapor.rs` | `compile_to_vapor` | 24      | 0       |
| TSX + modes  | `tests/parity_tsx.rs`   | both               | 10      | 0       |

The lower-level IR-shape tests (`tests/elements.rs`, `attributes.rs`,
`children.rs`, `components.rs`, `slots.rs`, `control_flow.rs`, `events.rs`,
`directives.rs`, `modes.rs`, `tsx.rs`, `vdom.rs`, `vapor.rs`) remain the
fine-grained lowering coverage; this inventory tracks the **parity** layer.

## Croquis reuse inventory (#1579)

JSX/TSX intentionally parses with OXC in the JSX dialect, then hands the parsed
`Program` to Croquis rather than reparsing as plain TypeScript. This keeps
binding, scope, macro, import, reactivity, and virtual-TS metadata owned by
Croquis while allowing JSX syntax to remain valid.

| JSX surface                     | Croquis reuse point                                                                            | Coverage / measurement                                     |
| ------------------------------- | ---------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| Script semantic analysis        | `Drawer::draw_script_setup_program` over the parsed OXC `Program`                              | `tests/analysis.rs`, `jsx_croquis_analyze` criterion group |
| Backend transform metadata      | `LowerOutput.analysis` shared with VDOM, Vapor, SSR, Canon, and Patina                         | `tests/compile.rs`, `tests/vdom.rs`, `tests/vapor.rs`      |
| Template expression identifiers | Core transform consumes the Croquis binding metadata instead of JSX-specific binding inference | backend parity snapshots                                   |

Intentional divergences:

| Surface                           | Reason                                                                                                                                                                                                                                |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| JSX component tag classification  | Vue template analysis treats unknown lowercase tags as components in some contexts; JSX follows the Babel/Vue JSX convention where lowercase element names are intrinsic/custom elements and capitalized/member names are components. |
| `.map(...)` control flow lowering | The JSX source is already an OXC callback AST, so alias spans come directly from callback parameter patterns instead of reserializing through Croquis's string-based `v-for` parser.                                                  |

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
|                  | `v-model={[val, ['trim']]}` modifier-array → `modelModifiers` `{ trim: true }`       |      ✅       |       |     |
|                  | `v-model={[val, 'foo', ['trim']]}` array arg+modifiers → `fooModifiers`              |      ✅       |       |     |
|                  | `v-model_lazy` / `v-model_number_lazy` suffix → v-model modifiers                    |      ✅       |       |     |
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

| Case               | Reason | Tracking |
| ------------------ | ------ | -------- |
| _(none currently)_ |        |          |

> Resolved in #1489: the `v-model` modifier-array form `{[val, ['trim']]}` and
> the `v-model_lazy` / `v-model_number_lazy` underscore-suffix form now lower to
> a `model` directive with `modelModifiers` + a single clean
> `onUpdate:modelValue` handler (see the `v_model_modifier_array_*` /
> `v_model_underscore_suffix_*` tests in `parity_vdom.rs`).

### Type-level / resolve-type parity — deferred

`@vue/babel-plugin-jsx`'s `resolveType` (deriving runtime props/emits from TS
type annotations) and broader type-level parity require the type-resolution
infrastructure that is not yet wired through this crate. Deferred pending the
type-checker work in **#1497 / #1502**. The current TSX suite covers _syntax_
acceptance and type-stripping, not type-driven prop/emit generation.

### Ecosystem references — documented manual workflow

Running the upstream `@vue/babel-plugin-jsx` + `vue-jsx-vapor` reference corpora
requires cloning external repos and network access, so it stays out of the fast,
offline PR CI lane. It is instead a
**documented manual workflow**: a pinned-revision manifest plus an `#[ignore]`d
coverage smoke.

- **Manifest** (`tests/ecosystem/testbeds.json`) — each entry pins a full commit
  SHA so reruns are deterministic; bump revisions deliberately.

  | Entry                 | Repo                     | Pinned revision |
  | --------------------- | ------------------------ | --------------- |
  | @vue/babel-plugin-jsx | `vuejs/babel-plugin-jsx` | `803aab3c…`     |
  | vue-jsx-vapor         | `vuejs/vue-jsx-vapor`    | `25eba175…`     |

Real-world component-library projects such as PrimeVue, Vuetify, and Naive UI
are tracked by the Vize-wide fixture registry
(`tests/_fixtures/vue-ecosystem-fixtures.json`) and run through the app e2e
check/lint lanes.

- **Run it**:

  ```text
  cargo test -p vize_atelier_jsx --test ecosystem_smoke -- --ignored --nocapture
  ```

  The smoke (`tests/ecosystem_smoke.rs`) shallow-clones each pinned entry, walks
  its roots for `.jsx`/`.tsx`, runs every file through `lower_source`, and prints
  a per-testbed table of files / clean / with-diagnostics / panicked. A panic
  fails the run — the compiler must surface a diagnostic, never unwind. Offline,
  the smoke degrades gracefully ("clone failed — skipped") instead of failing.

- **CI guard** — `tests/tooling/jsx-ecosystem-fixtures.test.ts` validates the
  manifest shape (pinned SHAs, github repos, roots, extensions) offline on every
  PR, keeping the testbed list honest without network access.

Wiring the smoke into a scheduled GitHub Actions lane (sharded, non-PR-gating)
is a maintainer infra decision, tracked alongside #1501's benchmark-CI lane.

## Performance benchmarks (#1501)

The JSX/TSX cost surface is benchmarked along **four** dimensions so a timing
regression points at the stage that caused it, not just "JSX got slower". All
four are A/B-compared (base vs head) in the `criterion-ab` GitHub Actions lane
(`.github/workflows/criterion-bench.yml`, via `bench/criterion-ab.mjs`), which
runs both the `vize_atelier_jsx` `jsx_compile` and the `vize_patina`
`markup_ir_bench` targets:

| Dimension             | Bench group(s)                                                    | Bench file                                | Measures                                           |
| --------------------- | ----------------------------------------------------------------- | ----------------------------------------- | -------------------------------------------------- |
| Parser / lowering     | `jsx_lower`                                                       | `vize_atelier_jsx/benches/jsx_compile.rs` | parse + lower to the shared relief IR              |
| Croquis analysis      | `jsx_croquis_analyze`                                             | `vize_atelier_jsx/benches/jsx_compile.rs` | binding/scope/reactivity, isolated (parse hoisted) |
| Patina rule traversal | `jsx_lint`                                                        | `vize_patina/benches/markup_ir_bench.rs`  | `Linter::lint_jsx` IR path vs lowering fallback    |
| VDOM / Vapor codegen  | `jsx_compile_vdom`, `jsx_compile_vapor`, `jsx_compile_mode_aware` | `vize_atelier_jsx/benches/jsx_compile.rs` | backend codegen for each output mode               |

**Regression threshold.** The lane is report-only by default (criterion is noisy
on shared runners). The documented JSX regression threshold is **10%**: setting
`CRITERION_AB_THRESHOLD: 10` in the workflow flips the lane into a hard gate that
fails on a +10% median regression on any of the ids above — see the `--threshold`
documentation in `bench/criterion-ab.mjs`.
