---
title: "Type and script rules"
description: "Checks for `<script>` and `<script setup>`: compiler macros, reactivity, props and emits, and rules that use TypeScript type information."
---

# Type and script rules

Checks for `<script>` and `<script setup>`: compiler macros, reactivity, props and emits, and rules that use TypeScript type information.

Each rule page shows a failing example, a passing example, and the configuration that turns the rule on.

## Essential (39)

Correctness rules in the `essential` preset.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="script-no-arrow-functions-in-watch"></span>[`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | Disallow arrow functions as Options API watch handlers. |  |
| <span id="script-no-async-in-computed"></span>[`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | Disallow async functions in computed properties. |  |
| <span id="script-no-computed-properties-in-data"></span>[`script/no-computed-properties-in-data`](./reference/script-no-computed-properties-in-data.md) | Disallow using computed properties inside the component data function. |  |
| <span id="script-no-deprecated-data-object-declaration"></span>[`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | Disallow an object literal as the component data option (Vue 3 requires a function). |  |
| <span id="script-no-deprecated-delete-set"></span>[`script/no-deprecated-delete-set`](./reference/script-no-deprecated-delete-set.md) | Disallow Vue 2 $set and $delete, which were removed in Vue 3. |  |
| <span id="script-no-deprecated-destroyed-lifecycle"></span>[`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | Disallow deprecated destroyed and beforeDestroy lifecycle hooks. | ✓ |
| <span id="script-no-deprecated-dollar-listeners-api"></span>[`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | Disallow the $listeners instance property removed in Vue 3 (merged into $attrs). |  |
| <span id="script-no-deprecated-dollar-scopedslots-api"></span>[`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | Disallow the $scopedSlots instance property removed in Vue 3 (use $slots). |  |
| <span id="script-no-deprecated-events-api"></span>[`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | Disallow the removed Vue 2 events API ($on / $off / $once). |  |
| <span id="script-no-deprecated-model-definition"></span>[`script/no-deprecated-model-definition`](./reference/script-no-deprecated-model-definition.md) | Disallow the Vue 2 model option, which was removed in Vue 3. |  |
| <span id="script-no-deprecated-props-default-this"></span>[`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | Disallow `this` inside a prop default/validator function (removed in Vue 3). |  |
| <span id="script-no-deprecated-vue-config-keycodes"></span>[`script/no-deprecated-vue-config-keycodes`](./reference/script-no-deprecated-vue-config-keycodes.md) | Disallow Vue.config.keyCodes, which was removed in Vue 3. |  |
| <span id="script-no-dupe-keys"></span>[`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | Disallow duplicate keys across Options API props/data/computed/methods/setup/inject. |  |
| <span id="script-no-duplicate-attr-inheritance"></span>[`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | Flag a component that applies its fallthrough attributes twice. |  |
| <span id="script-no-export-in-script-setup"></span>[`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | Disallow export statements inside &lt;script setup&gt;. |  |
| <span id="script-no-expose-after-await"></span>[`script/no-expose-after-await`](./reference/script-no-expose-after-await.md) | Disallow expose() and defineExpose() after an await. |  |
| <span id="script-no-import-compiler-macros"></span>[`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | Disallow importing Vue compiler macros that are auto-imported. |  |
| <span id="script-no-internal-imports"></span>[`script/no-internal-imports`](./reference/script-no-internal-imports.md) | Disallow importing from Vue internal modules. |  |
| <span id="script-no-lifecycle-after-await"></span>[`script/no-lifecycle-after-await`](./reference/script-no-lifecycle-after-await.md) | Disallow lifecycle hooks registered after an await in setup(). |  |
| <span id="script-no-potential-component-option-typo"></span>[`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | Flag likely typos in Options API component option names. |  |
| <span id="script-no-ref-as-operand"></span>[`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | Require ref-bound variables to be accessed via `.value` when used as an operand. |  |
| <span id="script-no-required-prop-with-default"></span>[`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | Disallow a prop that is both required: true and has a default. |  |
| <span id="script-no-reserved-identifiers"></span>[`script/no-reserved-identifiers`](./reference/script-no-reserved-identifiers.md) | Disallow using Vue compiler reserved identifiers. |  |
| <span id="script-no-reserved-keys"></span>[`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | Disallow Vue-reserved names as Options API props/data/computed/methods/setup keys. |  |
| <span id="script-no-reserved-props"></span>[`script/no-reserved-props`](./reference/script-no-reserved-props.md) | Disallow reserved names in a component's props declaration. |  |
| <span id="script-no-side-effects-in-computed-properties"></span>[`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | Disallow side effects in Options API computed getters. |  |
| <span id="script-no-use-computed-property-like-method"></span>[`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | Disallow calling an Options API computed property like a method. |  |
| <span id="script-no-watch-after-await"></span>[`script/no-watch-after-await`](./reference/script-no-watch-after-await.md) | Disallow watchers registered after an await in setup() without keeping their stop handle. |  |
| <span id="script-prefer-import-from-vue"></span>[`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | Prefer importing from 'vue' instead of internal packages. | ✓ |
| <span id="script-require-prop-type-constructor"></span>[`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | Require prop `type` values to be constructors rather than string literals. |  |
| <span id="script-require-render-return"></span>[`script/require-render-return`](./reference/script-require-render-return.md) | Require the component render function to return a value. |  |
| <span id="script-require-slots-as-functions"></span>[`script/require-slots-as-functions`](./reference/script-require-slots-as-functions.md) | Require $slots properties to be called as functions. |  |
| <span id="script-require-valid-default-prop"></span>[`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | Require a prop's default value to be valid for its declared type. |  |
| <span id="script-return-in-computed-property"></span>[`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | Require a return value in every computed getter. |  |
| <span id="script-return-in-emits-validator"></span>[`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | Require a return value in every Vue emits validator. |  |
| <span id="script-valid-define-emits"></span>[`script/valid-define-emits`](./reference/script-valid-define-emits.md) | Enforce valid defineEmits() usage (no type+runtime args, no local references, single call). |  |
| <span id="script-valid-define-options"></span>[`script/valid-define-options`](./reference/script-valid-define-options.md) | Enforce valid defineOptions() usage (single object arg, no props/emits/expose/slots). |  |
| <span id="script-valid-define-props"></span>[`script/valid-define-props`](./reference/script-valid-define-props.md) | Enforce valid defineProps() usage (single call, not both type and runtime args, no local references). |  |
| <span id="script-valid-next-tick"></span>[`script/valid-next-tick`](./reference/script-valid-next-tick.md) | Require the result of a nextTick() call to be awaited, chained, or given a callback. |  |

## Default (7)

Added by `happy-path`, the default preset.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="script-define-props-destructuring"></span>[`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | Enforce consistent style for defineProps destructuring in &lt;script setup&gt;. |  |
| <span id="script-no-multiple-slot-args"></span>[`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | Disallow passing more than one argument to a scoped-slot function call. |  |
| <span id="script-no-unstable-nested-components"></span>[`script/no-unstable-nested-components`](./reference/script-no-unstable-nested-components.md) | Disallow component definitions inside setup or render functions. |  |
| <span id="script-require-define-component"></span>[`script/require-define-component`](./reference/script-require-define-component.md) | Require defineComponent() for TypeScript SFC default exports. |  |
| <span id="script-require-explicit-emits"></span>[`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | Require emitted events to be declared in defineEmits or the emits option. |  |
| <span id="type-require-typed-emits"></span>[`type/require-typed-emits`](./reference/type-require-typed-emits.md) | Require type definition for defineEmits. |  |
| <span id="type-require-typed-props"></span>[`type/require-typed-props`](./reference/type-require-typed-props.md) | Require type definition for defineProps. |  |

## Stricter presets (22)

Only in `opinionated`, `nuxt` or `ecosystem`. The Presets column lists which.

| Rule | What it checks | Presets | Auto-fix |
| --- | --- | --- | --- |
| <span id="script-component-options-name-casing"></span>[`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | Enforce PascalCase for registered component names. | `opinionated`, `nuxt` |  |
| <span id="script-custom-event-name-casing"></span>[`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | Enforce configured casing for emitted custom event names. | `opinionated`, `nuxt` |  |
| <span id="script-define-emits-declaration"></span>[`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | Enforce the type-based defineEmits&lt;{}&gt;() form over the runtime/array form. | `opinionated`, `nuxt` |  |
| <span id="script-define-macros-order"></span>[`script/define-macros-order`](./reference/script-define-macros-order.md) | Enforce a consistent order of the Vue compiler macros in &lt;script setup&gt;. | `opinionated`, `nuxt` |  |
| <span id="script-define-props-declaration"></span>[`script/define-props-declaration`](./reference/script-define-props-declaration.md) | Enforce type-based defineProps&lt;{ ... }&gt;() over the runtime/object form. | `opinionated`, `nuxt` |  |
| <span id="script-no-boolean-default"></span>[`script/no-boolean-default`](./reference/script-no-boolean-default.md) | Disallow a default on a Boolean prop. | `opinionated`, `nuxt` |  |
| <span id="script-no-deep-destructure-in-props"></span>[`script/no-deep-destructure-in-props`](./reference/script-no-deep-destructure-in-props.md) | Disallow deeply nested destructuring in defineProps. | `opinionated`, `nuxt` |  |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Disallow getCurrentInstance() in Vapor mode (returns null). | `opinionated` |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | Disallow Options API patterns in Vapor mode. | `opinionated` |  |
| <span id="script-no-unused-emit-declarations"></span>[`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | Flag declared events that are never emitted. | `opinionated`, `nuxt` |  |
| <span id="script-no-with-defaults"></span>[`script/no-with-defaults`](./reference/script-no-with-defaults.md) | Discourage withDefaults in favor of destructuring defaults (Vue 3.5+). | `opinionated` |  |
| <span id="script-one-component-per-file"></span>[`script/one-component-per-file`](./reference/script-one-component-per-file.md) | Disallow more than one component definition in a file. | `opinionated`, `nuxt` |  |
| <span id="script-order-in-components"></span>[`script/order-in-components`](./reference/script-order-in-components.md) | Enforce the default order of properties in component options. | `opinionated`, `nuxt` |  |
| <span id="script-prefer-define-options"></span>[`script/prefer-define-options`](./reference/script-prefer-define-options.md) | Prefer defineOptions() over a default export paired with &lt;script setup&gt;. | `opinionated`, `nuxt` |  |
| <span id="script-prefer-use-template-ref"></span>[`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | Recommend useTemplateRef over ref for template references (Vue 3.5+). | `opinionated`, `nuxt` |  |
| <span id="script-require-default-prop"></span>[`script/require-default-prop`](./reference/script-require-default-prop.md) | Require a default value for every optional, non-Boolean prop. | `opinionated`, `nuxt` |  |
| <span id="script-require-explicit-slots"></span>[`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | Require slots consumed via useSlots() to be explicitly typed with defineSlots&lt;...&gt;(). | `opinionated`, `nuxt` |  |
| <span id="script-require-prop-types"></span>[`script/require-prop-types`](./reference/script-require-prop-types.md) | Require every prop to declare a type. | `opinionated` |  |
| <span id="script-require-typed-object-prop"></span>[`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | Require an explicit type on a prop whose runtime type is `Object` or `Array`. | `opinionated`, `nuxt` |  |
| <span id="type-no-floating-promises"></span>[`type/no-floating-promises`](./reference/type-no-floating-promises.md) | Disallow floating (unhandled) Promises. | `nuxt`, `opinionated` |  |
| <span id="type-no-reactivity-loss"></span>[`type/no-reactivity-loss`](./reference/type-no-reactivity-loss.md) | Disallow plain snapshots of reactive values across assignments and calls. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-template-binding"></span>[`type/no-unsafe-template-binding`](./reference/type-no-unsafe-template-binding.md) | Disallow template bindings that resolve to unsafe types. | `nuxt`, `opinionated` |  |

## Opt-in (16)

Not in any preset. Turn them on by name in the `linter.rules` passed to `withVue`.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="script-consistent-props-emits-key-quotes"></span>[`script/consistent-props-emits-key-quotes`](./reference/script-consistent-props-emits-key-quotes.md) | Require consistent quoted keys in props and emits declarations. |  |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | Disallow nextTick() usage in Vapor-oriented components. |  |
| <span id="script-no-reactive-destructure"></span>[`script/no-reactive-destructure`](./reference/script-no-reactive-destructure.md) | Disallow destructuring reactive objects which loses reactivity. |  |
| <span id="script-no-restricted-globals"></span>[`script/no-restricted-globals`](./reference/script-no-restricted-globals.md) | Disallow references to runtime-environment globals that must go through a typed wrapper. |  |
| <span id="script-no-restricted-members"></span>[`script/no-restricted-members`](./reference/script-no-restricted-members.md) | Disallow project-configured object.property member accesses. |  |
| <span id="script-no-top-level-ref-in-script"></span>[`script/no-top-level-ref-in-script`](./reference/script-no-top-level-ref-in-script.md) | Disallow top-level ref/reactive to prevent Cross-Request State Pollution. |  |
| <span id="script-prefer-computed"></span>[`script/prefer-computed`](./reference/script-prefer-computed.md) | Prefer computed() for derived reactive state. |  |
| <span id="script-prefer-define-emits-binding"></span>[`script/prefer-define-emits-binding`](./reference/script-prefer-define-emits-binding.md) | Prefer the captured defineEmits function over template $emit. | ✓ |
| <span id="script-prefer-ref-over-reactive"></span>[`script/prefer-ref-over-reactive`](./reference/script-prefer-ref-over-reactive.md) | Recommend using ref() over reactive() for state management. |  |
| <span id="script-prefer-use-attrs"></span>[`script/prefer-use-attrs`](./reference/script-prefer-use-attrs.md) | Recommend using useAttrs() over context.attrs. |  |
| <span id="script-prefer-use-id"></span>[`script/prefer-use-id`](./reference/script-prefer-use-id.md) | Recommend using useId() for generating unique IDs (Vue 3.5+). |  |
| <span id="script-prefer-use-slots"></span>[`script/prefer-use-slots`](./reference/script-prefer-use-slots.md) | Recommend using useSlots() over context.slots. |  |
| <span id="script-require-function-return-type"></span>[`script/require-function-return-type`](./reference/script-require-function-return-type.md) | Require return type annotations on functions. |  |
| <span id="script-require-symbol-provide"></span>[`script/require-symbol-provide`](./reference/script-require-symbol-provide.md) | Recommend using Symbol as injection key for provide/inject. |  |
| <span id="script-require-typed-ref"></span>[`script/require-typed-ref`](./reference/script-require-typed-ref.md) | Require an explicit type argument on a ref() initialized with no value, null, or undefined. |  |
| <span id="type-strict-boolean-expressions"></span>[`type/strict-boolean-expressions`](./reference/type-strict-boolean-expressions.md) | Require safe boolean expressions in script and template conditions. |  |

[Rules overview](../rules/index.md) · [Rule options](./options.md) · [Migrating from ESLint](./migration.md)
