---
title: Type and script rules
---

# Type and script rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Purpose |
| --- | --- |
| [`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | Enforce PascalCase for the component `name` option |
| [`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | Enforce camelCase for emitted custom event names |
| [`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | Enforce the type-based defineEmits&lt;{}&gt;() form over the runtime/array form |
| [`script/define-macros-order`](./reference/script-define-macros-order.md) | Enforce a consistent order of the Vue compiler macros in &lt;script setup&gt; |
| [`script/define-props-declaration`](./reference/script-define-props-declaration.md) | Enforce type-based defineProps&lt;{ ... }&gt;() over the runtime/object form |
| [`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | Enforce consistent style for defineProps destructuring in &lt;script setup&gt; |
| [`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | Disallow arrow functions as Options API watch handlers |
| [`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | Disallow async functions in computed properties |
| [`script/no-boolean-default`](./reference/script-no-boolean-default.md) | Disallow a default on a Boolean prop |
| [`script/no-deep-destructure-in-props`](./reference/script-no-deep-destructure-in-props.md) | Disallow deeply nested destructuring in defineProps |
| [`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | Disallow an object literal as the component data option (Vue 3 requires a function) |
| [`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | Disallow deprecated destroyed and beforeDestroy lifecycle hooks |
| [`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | Disallow the $listeners instance property removed in Vue 3 (merged into $attrs) |
| [`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | Disallow the $scopedSlots instance property removed in Vue 3 (use $slots) |
| [`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | Disallow the removed Vue 2 events API ($on / $off / $once) |
| [`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | Disallow `this` inside a prop default/validator function (removed in Vue 3) |
| [`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | Disallow duplicate keys across Options API props/data/computed/methods/setup/inject |
| [`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | Flag a component that applies its fallthrough attributes twice |
| [`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | Disallow export statements inside &lt;script setup&gt; |
| [`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Disallow getCurrentInstance() in Vapor mode (returns null) |
| [`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | Disallow importing Vue compiler macros that are auto-imported |
| [`script/no-internal-imports`](./reference/script-no-internal-imports.md) | Disallow importing from Vue internal modules |
| [`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | Disallow passing more than one argument to a scoped-slot function call |
| [`script/no-next-tick`](./reference/script-no-next-tick.md) | Disallow nextTick() usage in Vapor-oriented components |
| [`script/no-options-api`](./reference/script-no-options-api.md) | Disallow Options API patterns in Vapor mode |
| [`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | Flag likely typos in Options API component option names |
| [`script/no-reactive-destructure`](./reference/script-no-reactive-destructure.md) | Disallow destructuring reactive objects which loses reactivity |
| [`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | Require ref-bound variables to be accessed via `.value` when used as an operand |
| [`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | Disallow a prop that is both required: true and has a default |
| [`script/no-reserved-identifiers`](./reference/script-no-reserved-identifiers.md) | Disallow using Vue compiler reserved identifiers |
| [`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | Disallow Vue-reserved names as Options API props/data/computed/methods/setup/inject keys |
| [`script/no-reserved-props`](./reference/script-no-reserved-props.md) | Disallow reserved names in a component's props declaration |
| [`script/no-restricted-globals`](./reference/script-no-restricted-globals.md) | Disallow references to runtime-environment globals that must go through a typed wrapper |
| [`script/no-restricted-members`](./reference/script-no-restricted-members.md) | Disallow project-configured object.property member accesses |
| [`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | Disallow side effects in Options API computed getters |
| [`script/no-top-level-ref-in-script`](./reference/script-no-top-level-ref-in-script.md) | Disallow top-level ref/reactive to prevent Cross-Request State Pollution |
| [`script/no-unstable-nested-components`](./reference/script-no-unstable-nested-components.md) | Disallow component definitions inside setup or render functions |
| [`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | Flag declared events that are never emitted |
| [`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | Disallow calling an Options API computed property like a method |
| [`script/no-with-defaults`](./reference/script-no-with-defaults.md) | Discourage withDefaults in favor of destructuring defaults (Vue 3.5+) |
| [`script/prefer-computed`](./reference/script-prefer-computed.md) | Prefer computed() for derived reactive state |
| [`script/prefer-define-options`](./reference/script-prefer-define-options.md) | Prefer defineOptions() over a plain &lt;script&gt; that only sets name/inheritAttrs |
| [`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | Prefer importing from 'vue' instead of internal packages |
| [`script/prefer-ref-over-reactive`](./reference/script-prefer-ref-over-reactive.md) | Recommend using ref() over reactive() for state management |
| [`script/prefer-use-attrs`](./reference/script-prefer-use-attrs.md) | Recommend using useAttrs() over context.attrs |
| [`script/prefer-use-id`](./reference/script-prefer-use-id.md) | Recommend using useId() for generating unique IDs (Vue 3.5+) |
| [`script/prefer-use-slots`](./reference/script-prefer-use-slots.md) | Recommend using useSlots() over context.slots |
| [`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | Recommend useTemplateRef over ref for template references (Vue 3.5+) |
| [`script/require-default-prop`](./reference/script-require-default-prop.md) | Require a default value for every optional, non-Boolean prop |
| [`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | Require emitted events to be declared in defineEmits or the emits option |
| [`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | Require slots consumed via useSlots() to be explicitly typed with defineSlots&lt;...&gt;() |
| [`script/require-function-return-type`](./reference/script-require-function-return-type.md) | Require return type annotations on functions |
| [`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | Require prop `type` values to be constructors rather than string literals |
| [`script/require-prop-types`](./reference/script-require-prop-types.md) | Require every prop to declare a type |
| [`script/require-symbol-provide`](./reference/script-require-symbol-provide.md) | Recommend using Symbol as injection key for provide/inject |
| [`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | Require an explicit type on a prop whose runtime type is `Object` or `Array` |
| [`script/require-typed-ref`](./reference/script-require-typed-ref.md) | Require an explicit type argument on a ref() initialized with no value, null, or undefined |
| [`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | Require a prop's default value to be valid for its declared type |
| [`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | Require a return value in every computed getter |
| [`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | Require a return value in every Options API emits validator |
| [`script/valid-define-emits`](./reference/script-valid-define-emits.md) | Enforce valid defineEmits() usage (no type+runtime args, no local references, single call) |
| [`script/valid-define-options`](./reference/script-valid-define-options.md) | Enforce valid defineOptions() usage (single object arg, no props/emits/expose/slots) |
| [`script/valid-define-props`](./reference/script-valid-define-props.md) | Enforce valid defineProps() usage (single call, not both type and runtime args, no local references) |
| [`script/valid-next-tick`](./reference/script-valid-next-tick.md) | Require the result of a nextTick() call to be awaited, chained, or given a callback |
| [`type/no-floating-promises`](./reference/type-no-floating-promises.md) | Disallow floating (unhandled) Promises |
| [`type/no-reactivity-loss`](./reference/type-no-reactivity-loss.md) | Disallow plain snapshots of reactive values across assignments and calls |
| [`type/no-unsafe-template-binding`](./reference/type-no-unsafe-template-binding.md) | Disallow template bindings that resolve to unsafe types |
| [`type/require-typed-emits`](./reference/type-require-typed-emits.md) | Require type definition for defineEmits |
| [`type/require-typed-props`](./reference/type-require-typed-props.md) | Require type definition for defineProps |
| [`type/strict-boolean-expressions`](./reference/type-strict-boolean-expressions.md) | Require safe boolean expressions in script and template conditions |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
