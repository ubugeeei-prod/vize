---
title: "Type and script rules"
---

# Type and script rules

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`script/component-options-name-casing`](#script-component-options-name-casing) | [Bad](#script-component-options-name-casing-bad) · [Good](#script-component-options-name-casing-good) | Enforce PascalCase for the component `name` option |
| [`script/custom-event-name-casing`](#script-custom-event-name-casing) | [Bad](#script-custom-event-name-casing-bad) · [Good](#script-custom-event-name-casing-good) | Enforce camelCase for emitted custom event names |
| [`script/define-emits-declaration`](#script-define-emits-declaration) | [Bad](#script-define-emits-declaration-bad) · [Good](#script-define-emits-declaration-good) | Enforce the type-based defineEmits&lt;{}&gt;() form over the runtime/array form |
| [`script/define-macros-order`](#script-define-macros-order) | [Bad](#script-define-macros-order-bad) · [Good](#script-define-macros-order-good) | Enforce a consistent order of the Vue compiler macros in &lt;script setup&gt; |
| [`script/define-props-declaration`](#script-define-props-declaration) | [Bad](#script-define-props-declaration-bad) · [Good](#script-define-props-declaration-good) | Enforce type-based defineProps&lt;{ ... }&gt;() over the runtime/object form |
| [`script/define-props-destructuring`](#script-define-props-destructuring) | [Bad](#script-define-props-destructuring-bad) · [Good](#script-define-props-destructuring-good) | Enforce consistent style for defineProps destructuring in &lt;script setup&gt; |
| [`script/no-arrow-functions-in-watch`](#script-no-arrow-functions-in-watch) | [Bad](#script-no-arrow-functions-in-watch-bad) · [Good](#script-no-arrow-functions-in-watch-good) | Disallow arrow functions as Options API watch handlers |
| [`script/no-async-in-computed`](#script-no-async-in-computed) | [Bad](#script-no-async-in-computed-bad) · [Good](#script-no-async-in-computed-good) | Disallow async functions in computed properties |
| [`script/no-boolean-default`](#script-no-boolean-default) | [Bad](#script-no-boolean-default-bad) · [Good](#script-no-boolean-default-good) | Disallow a default on a Boolean prop |
| [`script/no-deep-destructure-in-props`](#script-no-deep-destructure-in-props) | [Bad](#script-no-deep-destructure-in-props-bad) · [Good](#script-no-deep-destructure-in-props-good) | Disallow deeply nested destructuring in defineProps |
| [`script/no-deprecated-data-object-declaration`](#script-no-deprecated-data-object-declaration) | [Bad](#script-no-deprecated-data-object-declaration-bad) · [Good](#script-no-deprecated-data-object-declaration-good) | Disallow an object literal as the component data option (Vue 3 requires a function) |
| [`script/no-deprecated-destroyed-lifecycle`](#script-no-deprecated-destroyed-lifecycle) | [Bad](#script-no-deprecated-destroyed-lifecycle-bad) · [Good](#script-no-deprecated-destroyed-lifecycle-good) | Disallow deprecated destroyed and beforeDestroy lifecycle hooks |
| [`script/no-deprecated-dollar-listeners-api`](#script-no-deprecated-dollar-listeners-api) | [Bad](#script-no-deprecated-dollar-listeners-api-bad) · [Good](#script-no-deprecated-dollar-listeners-api-good) | Disallow the $listeners instance property removed in Vue 3 (merged into $attrs) |
| [`script/no-deprecated-dollar-scopedslots-api`](#script-no-deprecated-dollar-scopedslots-api) | [Bad](#script-no-deprecated-dollar-scopedslots-api-bad) · [Good](#script-no-deprecated-dollar-scopedslots-api-good) | Disallow the $scopedSlots instance property removed in Vue 3 (use $slots) |
| [`script/no-deprecated-events-api`](#script-no-deprecated-events-api) | [Bad](#script-no-deprecated-events-api-bad) · [Good](#script-no-deprecated-events-api-good) | Disallow the removed Vue 2 events API ($on / $off / $once) |
| [`script/no-deprecated-props-default-this`](#script-no-deprecated-props-default-this) | [Bad](#script-no-deprecated-props-default-this-bad) · [Good](#script-no-deprecated-props-default-this-good) | Disallow `this` inside a prop default/validator function (removed in Vue 3) |
| [`script/no-dupe-keys`](#script-no-dupe-keys) | [Bad](#script-no-dupe-keys-bad) · [Good](#script-no-dupe-keys-good) | Disallow duplicate keys across Options API props/data/computed/methods/setup/inject |
| [`script/no-duplicate-attr-inheritance`](#script-no-duplicate-attr-inheritance) | [Bad](#script-no-duplicate-attr-inheritance-bad) · [Good](#script-no-duplicate-attr-inheritance-good) | Flag a component that applies its fallthrough attributes twice |
| [`script/no-export-in-script-setup`](#script-no-export-in-script-setup) | [Bad](#script-no-export-in-script-setup-bad) · [Good](#script-no-export-in-script-setup-good) | Disallow export statements inside &lt;script setup&gt; |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [Bad](#script-no-get-current-instance-bad) · [Good](#script-no-get-current-instance-good) | Disallow getCurrentInstance() in Vapor mode (returns null) |
| [`script/no-import-compiler-macros`](#script-no-import-compiler-macros) | [Bad](#script-no-import-compiler-macros-bad) · [Good](#script-no-import-compiler-macros-good) | Disallow importing Vue compiler macros that are auto-imported |
| [`script/no-internal-imports`](#script-no-internal-imports) | [Bad](#script-no-internal-imports-bad) · [Good](#script-no-internal-imports-good) | Disallow importing from Vue internal modules |
| [`script/no-multiple-slot-args`](#script-no-multiple-slot-args) | [Bad](#script-no-multiple-slot-args-bad) · [Good](#script-no-multiple-slot-args-good) | Disallow passing more than one argument to a scoped-slot function call |
| [`script/no-next-tick`](#script-no-next-tick) | [Bad](#script-no-next-tick-bad) · [Good](#script-no-next-tick-good) | Disallow nextTick() usage in Vapor-oriented components |
| [`script/no-options-api`](#script-no-options-api) | [Bad](#script-no-options-api-bad) · [Good](#script-no-options-api-good) | Disallow Options API patterns in Vapor mode |
| [`script/no-potential-component-option-typo`](#script-no-potential-component-option-typo) | [Bad](#script-no-potential-component-option-typo-bad) · [Good](#script-no-potential-component-option-typo-good) | Flag likely typos in Options API component option names |
| [`script/no-reactive-destructure`](#script-no-reactive-destructure) | [Bad](#script-no-reactive-destructure-bad) · [Good](#script-no-reactive-destructure-good) | Disallow destructuring reactive objects which loses reactivity |
| [`script/no-ref-as-operand`](#script-no-ref-as-operand) | [Bad](#script-no-ref-as-operand-bad) · [Good](#script-no-ref-as-operand-good) | Require ref-bound variables to be accessed via `.value` when used as an operand |
| [`script/no-required-prop-with-default`](#script-no-required-prop-with-default) | [Bad](#script-no-required-prop-with-default-bad) · [Good](#script-no-required-prop-with-default-good) | Disallow a prop that is both required: true and has a default |
| [`script/no-reserved-identifiers`](#script-no-reserved-identifiers) | [Bad](#script-no-reserved-identifiers-bad) · [Good](#script-no-reserved-identifiers-good) | Disallow using Vue compiler reserved identifiers |
| [`script/no-reserved-keys`](#script-no-reserved-keys) | [Bad](#script-no-reserved-keys-bad) · [Good](#script-no-reserved-keys-good) | Disallow Vue-reserved names as Options API props/data/computed/methods/setup/inject keys |
| [`script/no-reserved-props`](#script-no-reserved-props) | [Bad](#script-no-reserved-props-bad) · [Good](#script-no-reserved-props-good) | Disallow reserved names in a component's props declaration |
| [`script/no-restricted-globals`](#script-no-restricted-globals) | [Bad](#script-no-restricted-globals-bad) · [Good](#script-no-restricted-globals-good) | Disallow references to runtime-environment globals that must go through a typed wrapper |
| [`script/no-restricted-members`](#script-no-restricted-members) | [Bad](#script-no-restricted-members-bad) · [Good](#script-no-restricted-members-good) | Disallow project-configured object.property member accesses |
| [`script/no-side-effects-in-computed-properties`](#script-no-side-effects-in-computed-properties) | [Bad](#script-no-side-effects-in-computed-properties-bad) · [Good](#script-no-side-effects-in-computed-properties-good) | Disallow side effects in Options API computed getters |
| [`script/no-top-level-ref-in-script`](#script-no-top-level-ref-in-script) | [Bad](#script-no-top-level-ref-in-script-bad) · [Good](#script-no-top-level-ref-in-script-good) | Disallow top-level ref/reactive to prevent Cross-Request State Pollution |
| [`script/no-unstable-nested-components`](#script-no-unstable-nested-components) | [Bad](#script-no-unstable-nested-components-bad) · [Good](#script-no-unstable-nested-components-good) | Disallow component definitions inside setup or render functions |
| [`script/no-unused-emit-declarations`](#script-no-unused-emit-declarations) | [Bad](#script-no-unused-emit-declarations-bad) · [Good](#script-no-unused-emit-declarations-good) | Flag declared events that are never emitted |
| [`script/no-use-computed-property-like-method`](#script-no-use-computed-property-like-method) | [Bad](#script-no-use-computed-property-like-method-bad) · [Good](#script-no-use-computed-property-like-method-good) | Disallow calling an Options API computed property like a method |
| [`script/no-with-defaults`](#script-no-with-defaults) | [Bad](#script-no-with-defaults-bad) · [Good](#script-no-with-defaults-good) | Discourage withDefaults in favor of destructuring defaults (Vue 3.5+) |
| [`script/prefer-computed`](#script-prefer-computed) | [Bad](#script-prefer-computed-bad) · [Good](#script-prefer-computed-good) | Prefer computed() for derived reactive state |
| [`script/prefer-define-options`](#script-prefer-define-options) | [Bad](#script-prefer-define-options-bad) · [Good](#script-prefer-define-options-good) | Prefer defineOptions() over a plain &lt;script&gt; that only sets name/inheritAttrs |
| [`script/prefer-import-from-vue`](#script-prefer-import-from-vue) | [Bad](#script-prefer-import-from-vue-bad) · [Good](#script-prefer-import-from-vue-good) | Prefer importing from 'vue' instead of internal packages |
| [`script/prefer-ref-over-reactive`](#script-prefer-ref-over-reactive) | [Bad](#script-prefer-ref-over-reactive-bad) · [Good](#script-prefer-ref-over-reactive-good) | Recommend using ref() over reactive() for state management |
| [`script/prefer-use-attrs`](#script-prefer-use-attrs) | [Bad](#script-prefer-use-attrs-bad) · [Good](#script-prefer-use-attrs-good) | Recommend using useAttrs() over context.attrs |
| [`script/prefer-use-id`](#script-prefer-use-id) | [Bad](#script-prefer-use-id-bad) · [Good](#script-prefer-use-id-good) | Recommend using useId() for generating unique IDs (Vue 3.5+) |
| [`script/prefer-use-slots`](#script-prefer-use-slots) | [Bad](#script-prefer-use-slots-bad) · [Good](#script-prefer-use-slots-good) | Recommend using useSlots() over context.slots |
| [`script/prefer-use-template-ref`](#script-prefer-use-template-ref) | [Bad](#script-prefer-use-template-ref-bad) · [Good](#script-prefer-use-template-ref-good) | Recommend useTemplateRef over ref for template references (Vue 3.5+) |
| [`script/require-default-prop`](#script-require-default-prop) | [Bad](#script-require-default-prop-bad) · [Good](#script-require-default-prop-good) | Require a default value for every optional, non-Boolean prop |
| [`script/require-explicit-emits`](#script-require-explicit-emits) | [Bad](#script-require-explicit-emits-bad) · [Good](#script-require-explicit-emits-good) | Require emitted events to be declared in defineEmits or the emits option |
| [`script/require-explicit-slots`](#script-require-explicit-slots) | [Bad](#script-require-explicit-slots-bad) · [Good](#script-require-explicit-slots-good) | Require slots consumed via useSlots() to be explicitly typed with defineSlots&lt;...&gt;() |
| [`script/require-function-return-type`](#script-require-function-return-type) | [Bad](#script-require-function-return-type-bad) · [Good](#script-require-function-return-type-good) | Require return type annotations on functions |
| [`script/require-prop-type-constructor`](#script-require-prop-type-constructor) | [Bad](#script-require-prop-type-constructor-bad) · [Good](#script-require-prop-type-constructor-good) | Require prop `type` values to be constructors rather than string literals |
| [`script/require-prop-types`](#script-require-prop-types) | [Bad](#script-require-prop-types-bad) · [Good](#script-require-prop-types-good) | Require every prop to declare a type |
| [`script/require-symbol-provide`](#script-require-symbol-provide) | [Bad](#script-require-symbol-provide-bad) · [Good](#script-require-symbol-provide-good) | Recommend using Symbol as injection key for provide/inject |
| [`script/require-typed-object-prop`](#script-require-typed-object-prop) | [Bad](#script-require-typed-object-prop-bad) · [Good](#script-require-typed-object-prop-good) | Require an explicit type on a prop whose runtime type is `Object` or `Array` |
| [`script/require-typed-ref`](#script-require-typed-ref) | [Bad](#script-require-typed-ref-bad) · [Good](#script-require-typed-ref-good) | Require an explicit type argument on a ref() initialized with no value, null, or undefined |
| [`script/require-valid-default-prop`](#script-require-valid-default-prop) | [Bad](#script-require-valid-default-prop-bad) · [Good](#script-require-valid-default-prop-good) | Require a prop's default value to be valid for its declared type |
| [`script/return-in-computed-property`](#script-return-in-computed-property) | [Bad](#script-return-in-computed-property-bad) · [Good](#script-return-in-computed-property-good) | Require a return value in every computed getter |
| [`script/return-in-emits-validator`](#script-return-in-emits-validator) | [Bad](#script-return-in-emits-validator-bad) · [Good](#script-return-in-emits-validator-good) | Require a return value in every Options API emits validator |
| [`script/valid-define-emits`](#script-valid-define-emits) | [Bad](#script-valid-define-emits-bad) · [Good](#script-valid-define-emits-good) | Enforce valid defineEmits() usage (no type+runtime args, no local references, single call) |
| [`script/valid-define-options`](#script-valid-define-options) | [Bad](#script-valid-define-options-bad) · [Good](#script-valid-define-options-good) | Enforce valid defineOptions() usage (single object arg, no props/emits/expose/slots) |
| [`script/valid-define-props`](#script-valid-define-props) | [Bad](#script-valid-define-props-bad) · [Good](#script-valid-define-props-good) | Enforce valid defineProps() usage (single call, not both type and runtime args, no local references) |
| [`script/valid-next-tick`](#script-valid-next-tick) | [Bad](#script-valid-next-tick-bad) · [Good](#script-valid-next-tick-good) | Require the result of a nextTick() call to be awaited, chained, or given a callback |
| [`type/no-floating-promises`](#type-no-floating-promises) | [Bad](#type-no-floating-promises-bad) · [Good](#type-no-floating-promises-good) | Disallow floating (unhandled) Promises |
| [`type/no-reactivity-loss`](#type-no-reactivity-loss) | [Bad](#type-no-reactivity-loss-bad) · [Good](#type-no-reactivity-loss-good) | Disallow plain snapshots of reactive values across assignments and calls |
| [`type/no-unsafe-template-binding`](#type-no-unsafe-template-binding) | [Bad](#type-no-unsafe-template-binding-bad) · [Good](#type-no-unsafe-template-binding-good) | Disallow template bindings that resolve to unsafe types |
| [`type/require-typed-emits`](#type-require-typed-emits) | [Bad](#type-require-typed-emits-bad) · [Good](#type-require-typed-emits-good) | Require type definition for defineEmits |
| [`type/require-typed-props`](#type-require-typed-props) | [Bad](#type-require-typed-props-bad) · [Good](#type-require-typed-props-good) | Require type definition for defineProps |
| [`type/strict-boolean-expressions`](#type-strict-boolean-expressions) | [Bad](#type-strict-boolean-expressions-bad) · [Good](#type-strict-boolean-expressions-good) | Require safe boolean expressions in script and template conditions |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `script/component-options-name-casing`

Enforce PascalCase for the component `name` option

[Bad](#script-component-options-name-casing-bad) · [Good](#script-component-options-name-casing-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-component-options-name-casing-bad"></span>

**Bad**

The component option `name: 'my-component'` is kebab-case, whereas this rule requires a literal component name in PascalCase.

```vue annotate="remove:3"
<script lang="ts">
export default {
  name: 'my-component' // kebab-case
}
</script>
```

<span id="script-component-options-name-casing-good"></span>

**Good**

`MyComponent` begins with an uppercase letter and contains only alphanumeric characters, satisfying the name check.

```vue annotate="add:3"
<script lang="ts">
export default {
  name: 'MyComponent'
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) · [All rules](all.md)

### `script/custom-event-name-casing`

Enforce camelCase for emitted custom event names

[Bad](#script-custom-event-name-casing-bad) · [Good](#script-custom-event-name-casing-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: See [typed options and defaults](options.md).

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/custom-event-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-custom-event-name-casing-bad"></span>

**Bad**

The emitted string `my-event` contains a hyphen and violates the default camelCase event naming policy.

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits(['my-event'])
emit('my-event')         // kebab-case → report
</script>
```

<span id="script-custom-event-name-casing-good"></span>

**Good**

Both the declaration and call use `myEvent`, preserving agreement between the event name and its emission while satisfying the default casing policy. A configured kebab-case policy has a different expectation.

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits(['myEvent'])
emit('myEvent')
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61) · [All rules](all.md)

### `script/define-emits-declaration`

Enforce the type-based defineEmits&lt;{}&gt;() form over the runtime/array form

[Bad](#script-define-emits-declaration-bad) · [Good](#script-define-emits-declaration-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-emits-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-emits-declaration-bad"></span>

**Bad**

`defineEmits(["change"])` uses a runtime array declaration; this style rule prefers a type-based declaration.

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

<span id="script-define-emits-declaration-good"></span>

**Good**

`defineEmits<{ change: [id: number] }>()` moves the event declaration into a type argument and explicitly describes the numeric payload used by `emit("change", 1)`.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) · [All rules](all.md)

### `script/define-macros-order`

Enforce a consistent order of the Vue compiler macros in &lt;script setup&gt;

[Bad](#script-define-macros-order-bad) · [Good](#script-define-macros-order-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-macros-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-macros-order-bad"></span>

**Bad**

`defineProps` appears before `defineModel`, although `defineModel` has the earlier rank in the canonical macro order.

```vue annotate="remove:2,3"
<script setup lang="ts">
// defineProps before defineModel (out of canonical order)
const props = defineProps<{ count: number }>()
const model = defineModel<string>()
</script>
```

<span id="script-define-macros-order-good"></span>

**Good**

The declarations follow the exact sequence `defineOptions`, `defineModel`, `defineProps`, `defineEmits`, `defineSlots`, before unrelated runtime statements.

```vue annotate="add:2,4,5,6"
<script setup lang="ts">
defineOptions({ name: 'MyComponent' })
const model = defineModel<string>()
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
defineSlots<{ default(props: {}): any }>()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L46) · [All rules](all.md)

### `script/define-props-declaration`

Enforce type-based defineProps&lt;{ ... }&gt;() over the runtime/object form

[Bad](#script-define-props-declaration-bad) · [Good](#script-define-props-declaration-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-declaration-bad"></span>

**Bad**

`defineProps({ title: String })` supplies a runtime object, which conflicts with this rule’s preference for type-based props.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

<span id="script-define-props-declaration-good"></span>

**Good**

`defineProps<{ title: string }>()` declares `title` in the type argument and retains the `props.title` access without a runtime declaration argument.

```vue annotate="add:2"
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) · [All rules](all.md)

### `script/define-props-destructuring`

Enforce consistent style for defineProps destructuring in &lt;script setup&gt;

[Bad](#script-define-props-destructuring-bad) · [Good](#script-define-props-destructuring-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: See [typed options and defaults](options.md).

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-destructuring": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-destructuring-bad"></span>

**Bad**

`defineProps` is assigned to the single `props` binding rather than destructured, contrary to the default destructuring preference.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

<span id="script-define-props-destructuring-good"></span>

**Good**

The object pattern binds `foo` and `bar` directly and gives the optional `bar` a default. This relies on Vue 3.5+ reactive props destructuring; the configurable `never` mode prefers the opposite form.

```vue annotate="add:2"
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) · [All rules](all.md)

### `script/no-arrow-functions-in-watch`

Disallow arrow functions as Options API watch handlers

[Bad](#script-no-arrow-functions-in-watch-bad) · [Good](#script-no-arrow-functions-in-watch-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-arrow-functions-in-watch": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-arrow-functions-in-watch-bad"></span>

**Bad**

The Options API watcher `value` and the nested `other.handler` are arrow functions. An arrow captures its surrounding `this` rather than receiving the component instance.

```vue annotate="remove:4,5,9"
<script lang="ts">
export default {
  watch: {
    // `this` is not the component instance inside an arrow function.
    value: () => {
      this.doSomething()
    },
    other: {
      handler: () => {}
    }
  }
}
</script>
```

<span id="script-no-arrow-functions-in-watch-good"></span>

**Good**

Both handlers become ordinary methods, allowing Vue to bind `this` to the component. The `deep: true` watcher option remains compatible with the object form.

```vue annotate="add:4,8,9"
<script lang="ts">
export default {
  watch: {
    value(newValue, oldValue) {
      this.doSomething()
    },
    other: {
      handler(newValue) {},
      deep: true
    }
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59) · [All rules](all.md)

### `script/no-async-in-computed`

Disallow async functions in computed properties

[Bad](#script-no-async-in-computed-bad) · [Good](#script-no-async-in-computed-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-async-in-computed": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-async-in-computed-bad"></span>

**Bad**

The `computed` getter is `async`, so the fetch produces a Promise instead of a synchronously derived computed value.

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
import { computed } from "vue";
const data = computed(async () => {
  const response = await fetch("/api/data");
  return response.json();
});
</script>
```

<span id="script-no-async-in-computed-good"></span>

**Good**

The asynchronous fetch moves into `watch` and stores its result in `data.value`. Cleanup aborts the old request and prevents an inactive callback from writing a stale result; no async computed getter remains.

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
import { ref, watch } from "vue";
const query = ref("");
const data = ref<unknown>(null);
watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;
  onCleanup(() => { active = false; controller.abort(); });
  const response = await fetch(`/api/data?q=${encodeURIComponent(value)}`, { signal: controller.signal });
  const next: unknown = await response.json();
  if (active) data.value = next;
});
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L46) · [All rules](all.md)

### `script/no-boolean-default`

Disallow a default on a Boolean prop

[Bad](#script-no-boolean-default-bad) · [Good](#script-no-boolean-default-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-boolean-default": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-boolean-default-bad"></span>

**Bad**

Both `disabled` and `checked` declare a `default` on a prop whose sole constructor is `Boolean`; the rule rejects even an explicit `false` default.

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
  props: {
    // Boolean props already default to false; an explicit default is confusing.
    disabled: { type: Boolean, default: true },
    checked: { type: Boolean, default: false }
  }
}
</script>
```

<span id="script-no-boolean-default-good"></span>

**Good**

The Boolean-only props omit `default`, using Vue’s implicit false value. The `[Boolean, String]` union and the Number prop illustrate that this check is limited to the sole `Boolean` constructor.

```vue annotate="add:4,5,6,7,8,9,10"
<script lang="ts">
export default {
  props: {
    // No explicit default: defaults to false.
    disabled: { type: Boolean },
    disabled2: Boolean,
    // Union type may legitimately need a default.
    value: { type: [Boolean, String], default: '' },
    // Non-Boolean prop.
    count: { type: Number, default: 0 }
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54) · [All rules](all.md)

### `script/no-deep-destructure-in-props`

Disallow deeply nested destructuring in defineProps

[Bad](#script-no-deep-destructure-in-props-bad) · [Good](#script-no-deep-destructure-in-props-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deep-destructure-in-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deep-destructure-in-props-bad"></span>

**Bad**

The binding pattern descends through `user` to destructure `name`, exceeding the default shallow props-destructuring depth.

```vue annotate="remove:2"
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

<span id="script-no-deep-destructure-in-props-good"></span>

**Good**

The props object remains intact, and a computed getter reads `props.user.name`. The nested access stays explicit without a deeply nested binding pattern.

```vue annotate="add:2,3,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) · [All rules](all.md)

### `script/no-deprecated-data-object-declaration`

Disallow an object literal as the component data option (Vue 3 requires a function)

[Bad](#script-no-deprecated-data-object-declaration-bad) · [Good](#script-no-deprecated-data-object-declaration-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-data-object-declaration": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-data-object-declaration-bad"></span>

**Bad**

The Options API `data` option is an object literal, a Vue 2 form that Vue 3 no longer accepts.

```vue annotate="remove:3,4,5"
<script lang="ts">
export default {
  // `data` must be a function in Vue 3, not an object literal.
  data: {
    count: 0
  }
}
</script>
```

<span id="script-no-deprecated-data-object-declaration-good"></span>

**Good**

`data()` returns a new `{ count: 0 }` object, providing the function-based data declaration required by Vue 3.

```vue annotate="add:3,4"
<script lang="ts">
export default {
  data() {
    return { count: 0 }
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) · [All rules](all.md)

### `script/no-deprecated-destroyed-lifecycle`

Disallow deprecated destroyed and beforeDestroy lifecycle hooks

[Bad](#script-no-deprecated-destroyed-lifecycle-bad) · [Good](#script-no-deprecated-destroyed-lifecycle-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: Available for supported findings  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-destroyed-lifecycle": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-destroyed-lifecycle-bad"></span>

**Bad**

`beforeDestroy` is the removed Vue 2 lifecycle option used for the timer cleanup.

```vue annotate="remove:2"
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

**Good**

Renaming the hook to `beforeUnmount` preserves the cleanup body under its Vue 3 lifecycle name.

```vue annotate="add:2"
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) · [All rules](all.md)

### `script/no-deprecated-dollar-listeners-api`

Disallow the $listeners instance property removed in Vue 3 (merged into $attrs)

[Bad](#script-no-deprecated-dollar-listeners-api-bad) · [Good](#script-no-deprecated-dollar-listeners-api-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-listeners-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-listeners-api-bad"></span>

**Bad**

The member reads and the bare argument reference all use `$listeners`, which Vue 3 removed after merging listeners into attributes.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const handlers = this.$listeners
const forwarded = ctx.$listeners
emit('input', $listeners)
</script>
```

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

**Good**

The reads move to `this.$attrs` and setup-context `ctx.attrs`. These replace the removed listener surface; the illustrated receivers must exist in the surrounding component context.

```vue annotate="add:2,3"
<script setup lang="ts">
const handlers = this.$attrs
const forwarded = ctx.attrs
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L40) · [All rules](all.md)

### `script/no-deprecated-dollar-scopedslots-api`

Disallow the $scopedSlots instance property removed in Vue 3 (use $slots)

[Bad](#script-no-deprecated-dollar-scopedslots-api-bad) · [Good](#script-no-deprecated-dollar-scopedslots-api-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-scopedslots-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-scopedslots-api-bad"></span>

**Bad**

`this.$scopedSlots`, `ctx.$scopedSlots`, and the bare `$scopedSlots` reference use the Vue 2 scoped-slot API removed in Vue 3.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

**Good**

Replacing `$scopedSlots` with `$slots` uses the unified slot surface. The example removes the deprecated spelling rather than establishing a setup context for the receivers.

```vue annotate="add:2,3,4"
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L44) · [All rules](all.md)

### `script/no-deprecated-events-api`

Disallow the removed Vue 2 events API ($on / $off / $once)

[Bad](#script-no-deprecated-events-api-bad) · [Good](#script-no-deprecated-events-api-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-events-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-events-api-bad"></span>

**Bad**

The `$on`, `$once`, and `$off` calls use the instance event-bus methods removed in Vue 3.

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
this.$on('event', handler)
this.$once('event', handler)
this.$off('event', handler)
emitter.$off('event')
</script>
```

<span id="script-no-deprecated-events-api-good"></span>

**Good**

`$emit` remains valid, while event-bus subscription moves to the external emitter’s `on` method. The repair separates parent-directed emission from an external event bus.

```vue annotate="add:2,3,4,5,6,7,8"
<script setup lang="ts">
// $emit is still valid in Vue 3
this.$emit('event', payload)

// Use an external emitter instead
import mitt from 'mitt'
const emitter = mitt()
emitter.on('event', handler)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L42) · [All rules](all.md)

### `script/no-deprecated-props-default-this`

Disallow `this` inside a prop default/validator function (removed in Vue 3)

[Bad](#script-no-deprecated-props-default-this-bad) · [Good](#script-no-deprecated-props-default-this-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-props-default-this": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-props-default-this-bad"></span>

**Bad**

The prop default and validator read `this`, but those functions cannot rely on the component instance in Vue 3.

```vue annotate="remove:6,7,8,13,14"
<script lang="ts">
export default {
  props: {
    size: {
      type: Number,
      // `this` is not the component instance in Vue 3.
      default() {
        return this.defaultSize
      }
    },
    value: {
      type: Number,
      validator() {
        return this.value > 0
      }
    }
  }
}
</script>
```

<span id="script-no-deprecated-props-default-this-good"></span>

**Good**

The default reads `props.baseSize` from its argument, and the validator tests its `value` argument. Both stop depending on an unavailable instance receiver.

```vue annotate="add:6,7,8,13,14"
<script lang="ts">
export default {
  props: {
    size: {
      type: Number,
      // Vue 3 passes the raw props as the first argument instead.
      default(props) {
        return props.baseSize
      }
    },
    value: {
      type: Number,
      validator(value) {
        return value > 0
      }
    }
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) · [All rules](all.md)

### `script/no-dupe-keys`

Disallow duplicate keys across Options API props/data/computed/methods/setup/inject

[Bad](#script-no-dupe-keys-bad) · [Good](#script-no-dupe-keys-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-dupe-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-dupe-keys-bad"></span>

**Bad**

`foo` is declared by both props and data, and `bar` by both computed and methods. Those declarations compete for the same component instance keys.

```vue annotate="remove:5,8,9,10,11"
<script lang="ts">
export default {
  props: ['foo'],
  data() {
    return { foo: 1 } // duplicate of prop `foo`
  },
  computed: {
    bar() { return 2 }
  },
  methods: {
    bar() {} // duplicate of computed `bar`
  }
}
</script>
```

<span id="script-no-dupe-keys-good"></span>

**Good**

The prop, data, and computed declarations use distinct names (`foo`, `bar`, and `baz`), eliminating both cross-option collisions.

```vue annotate="add:5,8"
<script lang="ts">
export default {
  props: ['foo'],
  data() {
    return { bar: 1 }
  },
  computed: {
    baz() { return 2 }
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L52) · [All rules](all.md)

### `script/no-duplicate-attr-inheritance`

Flag a component that applies its fallthrough attributes twice

[Bad](#script-no-duplicate-attr-inheritance-bad) · [Good](#script-no-duplicate-attr-inheritance-good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-duplicate-attr-inheritance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-duplicate-attr-inheritance-bad"></span>

**Bad**

The explicit `inheritAttrs: true` values restate Vue’s default. This rule reports that redundant literal even when no root `$attrs` spread is shown.

```vue annotate="remove:2,3"
<script lang="ts">
defineOptions({ inheritAttrs: true })
export default { inheritAttrs: true }
</script>
```

<span id="script-no-duplicate-attr-inheritance-good"></span>

**Good**

`inheritAttrs: false` expresses a real opt-out, while the empty options object leaves default inheritance implicit. Neither restates the redundant `true` value.

```vue annotate="add:2,3"
<script lang="ts">
defineOptions({ inheritAttrs: false }) // intentional opt-out
export default {}                      // default inheritance, unstated
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L73) · [All rules](all.md)

### `script/no-export-in-script-setup`

Disallow export statements inside &lt;script setup&gt;

[Bad](#script-no-export-in-script-setup-bad) · [Good](#script-no-export-in-script-setup-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-export-in-script-setup": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-export-in-script-setup-bad"></span>

**Bad**

`export const count` attempts to expose a module export from `<script setup>`, where runtime exports are prohibited.

```vue annotate="remove:2"
<script setup lang="ts">
export const count = 1;
</script>
```

<span id="script-no-export-in-script-setup-good"></span>

**Good**

Removing `export` keeps `count` as a setup binding rather than a module export.

```vue annotate="add:2"
<script setup lang="ts">
const count = 1;
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) · [All rules](all.md)

### `script/no-get-current-instance`

Disallow getCurrentInstance() in Vapor mode (returns null)

[Bad](#script-no-get-current-instance-bad) · [Good](#script-no-get-current-instance-good)

Default severity: `error`  
Presets: `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-get-current-instance-bad"></span>

**Bad**

The Vapor-marked setup imports and calls `getCurrentInstance`, relying on an instance API this rule disallows for Vapor-oriented components.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

<span id="script-no-get-current-instance-good"></span>

**Good**

`inject("app-config")` obtains the explicitly provided configuration without importing or calling `getCurrentInstance`.

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [All rules](all.md)

### `script/no-import-compiler-macros`

Disallow importing Vue compiler macros that are auto-imported

[Bad](#script-no-import-compiler-macros-bad) · [Good](#script-no-import-compiler-macros-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-import-compiler-macros": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-import-compiler-macros-bad"></span>

**Bad**

The `vue` import includes `defineProps` and `defineEmits`, although these are compiler macros available directly in `<script setup>`.

```vue annotate="remove:2"
<script setup lang="ts">
import { defineProps, defineEmits } from "vue";
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

<span id="script-no-import-compiler-macros-good"></span>

**Good**

Removing the macro imports leaves both typed macro calls intact; no runtime import is needed for either declaration.

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) · [All rules](all.md)

### `script/no-internal-imports`

Disallow importing from Vue internal modules

[Bad](#script-no-internal-imports-bad) · [Good](#script-no-internal-imports-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-internal-imports": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-internal-imports-bad"></span>

**Bad**

Both imports address internal `dist` files rather than Vue’s public package entry point, coupling the component to build-file paths.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { foo } from '@vue/runtime-core/dist/runtime-core.esm-bundler'
import { bar } from 'vue/dist/vue.esm-bundler'
</script>
```

<span id="script-no-internal-imports-good"></span>

**Good**

Importing the required helpers from `vue` removes the dependency on internal distribution file locations.

```vue annotate="add:2"
<script setup lang="ts">
import { ref, computed } from 'vue'
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28) · [All rules](all.md)

### `script/no-multiple-slot-args`

Disallow passing more than one argument to a scoped-slot function call

[Bad](#script-no-multiple-slot-args-bad) · [Good](#script-no-multiple-slot-args-good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-multiple-slot-args": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-multiple-slot-args-bad"></span>

**Bad**

The slot calls pass multiple positional arguments or spread an unknown argument list. Vue slots receive one props object, not a positional parameter list.

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
slots.default(foo, bar)
$slots.header(a, b)
this.$scopedSlots.item(x, y)
useSlots().default(a, b)
slots.default(...args)
</script>
```

<span id="script-no-multiple-slot-args-good"></span>

**Good**

`{ foo, bar }` combines the data into one argument; `slotProps` and the argument-free call also stay within the supported slot-call shape.

```vue annotate="add:2,3,4"
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) · [All rules](all.md)

### `script/no-next-tick`

Disallow nextTick() usage in Vapor-oriented components

[Bad](#script-no-next-tick-bad) · [Good](#script-no-next-tick-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-next-tick-bad"></span>

**Bad**

The Vapor-oriented component imports and awaits `nextTick`, introducing the DOM-flush scheduling dependency that this migration rule rejects.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

<span id="script-no-next-tick-good"></span>

**Good**

The input is obtained through `useTemplateRef` and focused at `onMounted`. The explicit mount boundary replaces the example’s `nextTick` dependency.

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [All rules](all.md)

### `script/no-options-api`

Disallow Options API patterns in Vapor mode

[Bad](#script-no-options-api-bad) · [Good](#script-no-options-api-good)

Default severity: `error`  
Presets: `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-options-api-bad"></span>

**Bad**

The default-export object declares Options API `data()`, a component option form prohibited by this rule.

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

<span id="script-no-options-api-good"></span>

**Good**

The component state becomes a Composition API `ref` in Vapor `<script setup>`, removing the Options API object and its `data` option.

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [All rules](all.md)

### `script/no-potential-component-option-typo`

Flag likely typos in Options API component option names

[Bad](#script-no-potential-component-option-typo-bad) · [Good](#script-no-potential-component-option-typo-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-potential-component-option-typo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-potential-component-option-typo-bad"></span>

**Bad**

The option is spelled `method`, one edit away from the recognized `methods` option; Vue would not treat it as the intended methods declaration.

```vue annotate="remove:2"
<script lang="ts">
export default { method: { save() {} } };
</script>
```

<span id="script-no-potential-component-option-typo-good"></span>

**Good**

Changing the key to `methods` places `save()` under the recognized component option.

```vue annotate="add:2"
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) · [All rules](all.md)

### `script/no-reactive-destructure`

Disallow destructuring reactive objects which loses reactivity

[Bad](#script-no-reactive-destructure-bad) · [Good](#script-no-reactive-destructure-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reactive-destructure": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reactive-destructure-bad"></span>

**Bad**

`const { count, name } = state` copies primitive properties out of the `reactive` object, losing their connection to subsequent property changes.

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = state;
</script>
```

<span id="script-no-reactive-destructure-good"></span>

**Good**

Destructuring `toRefs(state)` creates refs for `count` and `name`, keeping each binding linked to the original reactive property.

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRefs } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = toRefs(state);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) · [All rules](all.md)

### `script/no-ref-as-operand`

Require ref-bound variables to be accessed via `.value` when used as an operand

[Bad](#script-no-ref-as-operand-bad) · [Good](#script-no-ref-as-operand-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-ref-as-operand": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-ref-as-operand-bad"></span>

**Bad**

`count + 1` uses the ref object itself as the arithmetic operand instead of the number it wraps.

```vue annotate="remove:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count + 1;
</script>
```

<span id="script-no-ref-as-operand-good"></span>

**Good**

`count.value + 1` reads the wrapped number before adding one; script arithmetic requires this explicit ref access.

```vue annotate="add:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count.value + 1;
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) · [All rules](all.md)

### `script/no-required-prop-with-default`

Disallow a prop that is both required: true and has a default

[Bad](#script-no-required-prop-with-default-bad) · [Good](#script-no-required-prop-with-default-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-required-prop-with-default": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-required-prop-with-default-bad"></span>

**Bad**

`title` is both required and given the fallback `"Untitled"`, combining a required-input contract with a default intended for missing input.

```vue annotate="remove:2"
<script lang="ts">
export default { props: { title: { type: String, required: true, default: "Untitled" } } };
</script>
```

<span id="script-no-required-prop-with-default-good"></span>

**Good**

Removing `required: true` makes `title` optional and leaves `"Untitled"` as its coherent fallback.

```vue annotate="add:2"
<script lang="ts">
export default { props: { title: { type: String, default: "Untitled" } } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L29) · [All rules](all.md)

### `script/no-reserved-identifiers`

Disallow using Vue compiler reserved identifiers

[Bad](#script-no-reserved-identifiers-bad) · [Good](#script-no-reserved-identifiers-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-identifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-identifiers-bad"></span>

**Bad**

The bindings `__props`, `__emit`, and `__sfc__` use identifiers reserved for generated Vue compiler code.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const __props = { name: "Ada" };
const __emit = () => {};
const __sfc__ = {};
</script>
```

<span id="script-no-reserved-identifiers-good"></span>

**Good**

The ordinary names `props`, `emit`, and `componentData` avoid those generated identifiers while retaining props and emits declarations.

```vue annotate="add:2,3,4"
<script setup lang="ts">
const props = defineProps<{ name: string }>();
const emit = defineEmits<{ save: [] }>();
const componentData = {};
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) · [All rules](all.md)

### `script/no-reserved-keys`

Disallow Vue-reserved names as Options API props/data/computed/methods/setup/inject keys

[Bad](#script-no-reserved-keys-bad) · [Good](#script-no-reserved-keys-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-keys-bad"></span>

**Bad**

The returned data key `$el` collides with Vue’s built-in component-instance property and also uses a reserved `$` prefix.

```vue annotate="remove:2"
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

<span id="script-no-reserved-keys-good"></span>

**Good**

Renaming the application data to `elementLabel` avoids the built-in instance surface and reserved prefix.

```vue annotate="add:2"
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L32) · [All rules](all.md)

### `script/no-reserved-props`

Disallow reserved names in a component's props declaration

[Bad](#script-no-reserved-props-bad) · [Good](#script-no-reserved-props-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-props-bad"></span>

**Bad**

The object-form `ref` and `$foo`, plus the array-form `key`, are reserved prop names. `ref` and `key` are framework controls, and `$`-prefixed names are rejected.

```vue annotate="remove:4,5,8,9,10,11"
<script lang="ts">
export default {
  props: {
    ref: String,   // reserved
    $foo: Number    // `$`-prefixed names are reserved
  }
}

export default {
  props: ['key']    // reserved (array form)
}
</script>
```

<span id="script-no-reserved-props-good"></span>

**Good**

The ordinary prop names `name` and `refValue` avoid the reserved names in both spelling and prefix.

```vue annotate="add:4,5"
<script lang="ts">
export default {
  props: {
    name: String,
    refValue: Number
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) · [All rules](all.md)

### `script/no-restricted-globals`

Disallow references to runtime-environment globals that must go through a typed wrapper

[Bad](#script-no-restricted-globals-bad) · [Good](#script-no-restricted-globals-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: See [typed options and defaults](options.md).

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-globals": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-restricted-globals-bad"></span>

**Bad**

The example reads the default restricted globals `process`, `localStorage`, and `sessionStorage` directly, bypassing the project’s explicit config and storage helpers.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const flag = process.env.FEATURE_FLAG
const token = localStorage.getItem('auth.token')
sessionStorage.setItem('view.scroll', String(window.scrollY))
</script>
```

<span id="script-no-restricted-globals-good"></span>

**Good**

`useFeatureFlag`, `authStorage.read`, and `viewStorage.write` remove those direct restricted-global references. The remaining `window.scrollY` is not a default restriction of this rule; SSR safety is a separate concern.

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
// Use a typed config helper that distinguishes server vs. client.
const flag = useFeatureFlag('FEATURE_FLAG')

// Use a typed wrapper that scopes keys and handles SSR / disabled storage.
const token = authStorage.read('auth.token')
viewStorage.write('view.scroll', String(window.scrollY))
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) · [All rules](all.md)

### `script/no-restricted-members`

Disallow project-configured object.property member accesses

[Bad](#script-no-restricted-members-bad) · [Good](#script-no-restricted-members-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: See [typed options and defaults](options.md).

This example configures window.localStorage. The rule has no default deny list; enabling it alone does not report a member.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-members": "error"
      },
      "ruleOptions": {
        "script/no-restricted-members": {
          "members": [
            {
              "object": "window",
              "property": "localStorage"
            }
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-restricted-members-bad"></span>

**Bad**

With `{ object: "window", property: "localStorage" }` configured in `ruleOptions`, `window.localStorage` accesses the forbidden object/member pair. This rule has no default forbidden members.

```vue annotate="remove:2"
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

<span id="script-no-restricted-members-good"></span>

**Good**

`authStorage.read("token")` delegates the read to the application’s storage helper and no longer accesses the configured `window.localStorage` member.

```vue annotate="add:2"
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) · [All rules](all.md)

### `script/no-side-effects-in-computed-properties`

Disallow side effects in Options API computed getters

[Bad](#script-no-side-effects-in-computed-properties-bad) · [Good](#script-no-side-effects-in-computed-properties-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-side-effects-in-computed-properties": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-side-effects-in-computed-properties-bad"></span>

**Bad**

`doubled` assigns to `this.count`, and `reversed` mutates `this.items` through `reverse()`. Both getters modify the state they are supposed to derive from.

```vue annotate="remove:8,9,12"
<script lang="ts">
export default {
  data() {
    return { count: 0, items: [] }
  },
  computed: {
    doubled() {
      this.count = this.count * 2 // side effect: assigns to data
      return this.count
    },
    reversed() {
      return this.items.reverse() // side effect: mutates the array
    }
  }
}
</script>
```

<span id="script-no-side-effects-in-computed-properties-good"></span>

**Good**

`doubled` returns the multiplication without assignment. `reversed` copies the array before reversing it, so the original component state is unchanged by the getter.

```vue annotate="add:8,11"
<script lang="ts">
export default {
  data() {
    return { count: 0, items: [] }
  },
  computed: {
    doubled() {
      return this.count * 2
    },
    reversed() {
      return [...this.items].reverse() // operate on a copy
    }
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L70) · [All rules](all.md)

### `script/no-top-level-ref-in-script`

Disallow top-level ref/reactive to prevent Cross-Request State Pollution

[Bad](#script-no-top-level-ref-in-script-bad) · [Good](#script-no-top-level-ref-in-script-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-top-level-ref-in-script": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-top-level-ref-in-script-bad"></span>

**Bad**

The ordinary `<script>` initializes `count` and `user` at module scope. During SSR, these state objects can be shared across component instances and requests.

```vue annotate="remove:1,2,4,8"
<script>
// This state is shared across all requests in SSR!
const count = ref(0)
const user = reactive({ name: '' })

export default {
  setup() {
    return { count, user }
  }
}
</script>
```

<span id="script-no-top-level-ref-in-script-good"></span>

**Good**

The setup ref is initialized per component instance; the ordinary script keeps only a constant, a state-producing function, and a ref created inside `setup()`. None creates reactive state at ordinary module scope.

```vue annotate="add:1,2,4,6,7,8,9,10,11,12,13,14,17,18,19"
<script setup>
// Script setup creates fresh state per request
const count = ref(0)
</script>

<script>
// Constants are fine
const API_URL = 'https://api.example.com'

// Functions that create state are fine
function createState() {
  return reactive({ count: 0 })
}

export default {
  setup() {
    // Create state inside setup
    const count = ref(0)
    return { count }
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) · [All rules](all.md)

### `script/no-unstable-nested-components`

Disallow component definitions inside setup or render functions

[Bad](#script-no-unstable-nested-components-bad) · [Good](#script-no-unstable-nested-components-good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-unstable-nested-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unstable-nested-components-bad"></span>

**Bad**

`defineComponent` runs inside the parent’s `setup()`, creating a new `Child` component definition whenever that setup executes.

```vue annotate="remove:3"
<script lang="ts">
import { defineComponent } from "vue";
export default { setup() { const Child = defineComponent({ render() { return null; } }); return { Child }; } };
</script>
```

<span id="script-no-unstable-nested-components-good"></span>

**Good**

The `Child` definition moves to module scope, and `setup()` returns that existing definition instead of recreating it.

```vue annotate="add:3,4"
<script lang="ts">
import { defineComponent } from "vue";
const Child = defineComponent({ render() { return null; } });
export default { setup() { return { Child }; } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) · [All rules](all.md)

### `script/no-unused-emit-declarations`

Flag declared events that are never emitted

[Bad](#script-no-unused-emit-declarations-bad) · [Good](#script-no-unused-emit-declarations-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-unused-emit-declarations": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unused-emit-declarations-bad"></span>

**Bad**

`defineEmits` declares both `change` and `unused`, but the captured `emit` function only emits the literal event `change`.

```vue annotate="remove:2,4"
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

<span id="script-no-unused-emit-declarations-good"></span>

**Good**

Removing `unused` makes the declared event list match the observed emission. The example uses a captured, unescaped emit binding so this local usage conclusion is available.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) · [All rules](all.md)

### `script/no-use-computed-property-like-method`

Disallow calling an Options API computed property like a method

[Bad](#script-no-use-computed-property-like-method-bad) · [Good](#script-no-use-computed-property-like-method-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-use-computed-property-like-method": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-use-computed-property-like-method-bad"></span>

**Bad**

`this.total()` calls the value exposed by the computed getter; the getter returns `3`, which is not callable.

```vue annotate="remove:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total()); } } };
</script>
```

<span id="script-no-use-computed-property-like-method-good"></span>

**Good**

`this.total` reads the computed value without call parentheses, so `log` prints the derived number.

```vue annotate="add:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total); } } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44) · [All rules](all.md)

### `script/no-with-defaults`

Discourage withDefaults in favor of destructuring defaults (Vue 3.5+)

[Bad](#script-no-with-defaults-bad) · [Good](#script-no-with-defaults-good)

Default severity: `warning`  
Presets: `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-with-defaults": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-with-defaults-bad"></span>

**Bad**

`withDefaults` wraps the typed props declaration solely to supply `count` and `name` defaults, instead of the Vue 3.5+ destructuring-default style preferred here.

```vue annotate="remove:2"
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

<span id="script-no-with-defaults-good"></span>

**Good**

The destructuring pattern puts `count = 0` and `name = "Ada"` beside their bindings and removes the `withDefaults` wrapper.

```vue annotate="add:2"
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) · [All rules](all.md)

### `script/prefer-computed`

Prefer computed() for derived reactive state

[Bad](#script-prefer-computed-bad) · [Good](#script-prefer-computed-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

The watcher must only derive the destination. Editable copies and callbacks with other side effects are allowed.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-computed": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-computed-bad"></span>

**Bad**

The watcher only copies a derivation of `count` into a second ref, `doubled`, so the derived state is maintained through manual synchronization.

```vue annotate="remove:2,4,5"
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
const doubled = ref(0);
watch(count, (value) => { doubled.value = value * 2; });
</script>
```

<span id="script-prefer-computed-good"></span>

**Good**

`computed(() => count.value * 2)` expresses the derivation directly and removes both the extra writable ref and its synchronization watcher.

```vue annotate="add:2,4"
<script setup lang="ts">
import { ref, computed } from "vue";
const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) · [All rules](all.md)

### `script/prefer-define-options`

Prefer defineOptions() over a plain &lt;script&gt; that only sets name/inheritAttrs

[Bad](#script-prefer-define-options-bad) · [Good](#script-prefer-define-options-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-define-options": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-define-options-bad"></span>

**Bad**

The plain script’s only meaningful statement exports an object containing just `name` and `inheritAttrs`; these options can be expressed by `defineOptions`.

```vue annotate="remove:2"
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

<span id="script-prefer-define-options-good"></span>

**Good**

The shown `data()` method makes the script carry real Options API logic, so it falls outside this rule’s conservative options-only suggestion. This Good demonstrates an allowed exception; the direct migration would put `defineOptions({ name: 'MyComponent', inheritAttrs: false })` in `<script setup>`.

```vue annotate="add:2,3,4,5,6"
<script lang="ts">
// Real options logic — keep the plain script.
export default {
  name: 'MyComponent',
  data() { return { count: 0 } },
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) · [All rules](all.md)

### `script/prefer-import-from-vue`

Prefer importing from 'vue' instead of internal packages

[Bad](#script-prefer-import-from-vue-bad) · [Good](#script-prefer-import-from-vue-good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-import-from-vue": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-import-from-vue-bad"></span>

**Bad**

`ref` and `h` are imported from the internal `@vue/runtime-core` and `@vue/runtime-dom` packages rather than the public `vue` package.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

<span id="script-prefer-import-from-vue-good"></span>

**Good**

Both helpers are imported together from `vue`, using the public package entry point instead of either internal package.

```vue annotate="add:2"
<script setup lang="ts">
import { ref, h } from 'vue'
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L32) · [All rules](all.md)

### `script/prefer-ref-over-reactive`

Recommend using ref() over reactive() for state management

[Bad](#script-prefer-ref-over-reactive-bad) · [Good](#script-prefer-ref-over-reactive-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-ref-over-reactive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-ref-over-reactive-bad"></span>

**Bad**

The state is created with `reactive`, contrary to this opinionated rule’s preference for refs. The example illustrates a style preference, not an inherently invalid reactive object.

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
// reactive requires careful handling to avoid losing reactivity
const state = reactive({
  count: 0,
  name: 'foo'
})
</script>
```

<span id="script-prefer-ref-over-reactive-good"></span>

**Good**

The examples create both scalar and object state with `ref`; related fields may also be split into separate refs. This satisfies the preferred state-construction form.

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
// ref is more explicit and safer
const count = ref(0)
const name = ref('foo')

// For objects, ref still works
const user = ref({ name: 'foo', age: 20 })

// Or use multiple refs for related data
const userName = ref('foo')
const userAge = ref(20)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) · [All rules](all.md)

### `script/prefer-use-attrs`

Recommend using useAttrs() over context.attrs

[Bad](#script-prefer-use-attrs-bad) · [Good](#script-prefer-use-attrs-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-attrs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-attrs-bad"></span>

**Bad**

`setup` obtains `attrs` by destructuring its context parameter, which this rule asks to replace with the Composition API helper.

```vue annotate="remove:2"
<script lang="ts">
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

<span id="script-prefer-use-attrs-good"></span>

**Good**

`useAttrs()` supplies `attrs` inside setup, retaining the `attrs.class` read without depending on the second setup parameter.

```vue annotate="add:2,3"
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) · [All rules](all.md)

### `script/prefer-use-id`

Recommend using useId() for generating unique IDs (Vue 3.5+)

[Bad](#script-prefer-use-id-bad) · [Good](#script-prefer-use-id-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-id-bad"></span>

**Bad**

`id` contains `Math.random()`, so the generated input/label identifier can differ between server and client rendering. Its ID-named binding is the rule’s recognized generation context.

```vue annotate="remove:2"
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

<span id="script-prefer-use-id-good"></span>

**Good**

Vue 3.5+ `useId()` generates the identifier, and both `:for` and `:id` continue reading the same binding instead of independently generating random values.

```vue annotate="add:2,3"
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) · [All rules](all.md)

### `script/prefer-use-slots`

Recommend using useSlots() over context.slots

[Bad](#script-prefer-use-slots-bad) · [Good](#script-prefer-use-slots-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-slots-bad"></span>

**Bad**

`setup` destructures `slots` from its context argument, the access form this rule prefers to replace.

```vue annotate="remove:2,4"
<script lang="ts">
import { defineComponent, h } from "vue";
export default defineComponent({
  setup(_props, { slots }) { return () => h("div", slots.default?.()); },
});
</script>
```

<span id="script-prefer-use-slots-good"></span>

**Good**

`useSlots()` retrieves the slots inside setup, preserving the render function and its optional default-slot call without a context parameter.

```vue annotate="add:2,4,5,6,7"
<script lang="ts">
import { defineComponent, h, useSlots } from "vue";
export default defineComponent({
  setup() {
    const slots = useSlots();
    return () => h("div", slots.default?.());
  },
});
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) · [All rules](all.md)

### `script/prefer-use-template-ref`

Recommend useTemplateRef over ref for template references (Vue 3.5+)

[Bad](#script-prefer-use-template-ref-bad) · [Good](#script-prefer-use-template-ref-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-template-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-template-ref-bad"></span>

**Bad**

The nullable `input` ref is paired with the template’s literal `ref="input"`, identifying it as an element reference rather than ordinary nullable data.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from 'vue'
const input = ref<HTMLInputElement | null>(null)
</script>
<template>
  <input ref="input" />
</template>
```

<span id="script-prefer-use-template-ref-good"></span>

**Good**

Vue 3.5+ `useTemplateRef<HTMLInputElement>('input')` makes that template reference explicit. The unpaired `error = ref(null)` remains ordinary data and is intentionally outside this rule.

```vue annotate="add:2,3,4,5,6,10"
<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'
// Paired with the template ref below.
const input = useTemplateRef<HTMLInputElement>('input')
// A nullable data ref the template never binds as a ref.
const error = ref(null)
</script>
<template>
  <input ref="input" />
  <p>{{ error }}</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75) · [All rules](all.md)

### `script/require-default-prop`

Require a default value for every optional, non-Boolean prop

[Bad](#script-require-default-prop-bad) · [Good](#script-require-default-prop-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-default-prop-bad"></span>

**Bad**

`name` and `age` are optional non-Boolean runtime props without defaults, leaving their omitted-input values unspecified.

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
  props: {
    // optional, non-Boolean, no default
    name: String,
    age: { type: Number },
  }
}
</script>
```

<span id="script-require-default-prop-good"></span>

**Good**

`name` receives `default: ''`. `enabled` uses Boolean’s implicit false default, and required `id` needs no fallback, illustrating both exemptions.

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
  props: {
    name: { type: String, default: '' },
    enabled: Boolean,                 // Boolean defaults to false
    id: { type: Number, required: true },
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) · [All rules](all.md)

### `script/require-explicit-emits`

Require emitted events to be declared in defineEmits or the emits option

[Bad](#script-require-explicit-emits-bad) · [Good](#script-require-explicit-emits-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-emits": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-emits-bad"></span>

**Bad**

The captured emit function emits `save`, but `defineEmits([])` declares no such event.

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits([]);
emit("save");
</script>
```

<span id="script-require-explicit-emits-good"></span>

**Good**

Adding `"save"` to the declaration makes the emitted literal event part of the component’s explicit event contract.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(["save"]);
emit("save");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L61) · [All rules](all.md)

### `script/require-explicit-slots`

Require slots consumed via useSlots() to be explicitly typed with defineSlots&lt;...&gt;()

[Bad](#script-require-explicit-slots-bad) · [Good](#script-require-explicit-slots-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-slots-bad"></span>

**Bad**

The typed `defineProps<{ id: number }>()` establishes TypeScript syntax, but setup uses `useSlots()` without a `defineSlots` declaration. The rule therefore finds consumed slots without an explicit slot contract.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ id: number }>()
const slots = useSlots()
</script>
```

<span id="script-require-explicit-slots-good"></span>

**Good**

`defineSlots` declares a `default` slot whose props include `msg: string`; `useSlots()` now appears alongside an explicit typed slot contract.

```vue annotate="add:2"
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) · [All rules](all.md)

### `script/require-function-return-type`

Require return type annotations on functions

[Bad](#script-require-function-return-type-bad) · [Good](#script-require-function-return-type-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-function-return-type": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-function-return-type-bad"></span>

**Bad**

Both `add` and `greet` annotate their parameters but omit a return-type annotation; inferred returns do not satisfy this explicit-annotation policy.

```vue annotate="remove:2,6"
<script setup lang="ts">
const add = (a: number, b: number) => {
  return a + b
}

function greet(name: string) {
  return `Hello, ${name}`
}
</script>
```

<span id="script-require-function-return-type-good"></span>

**Good**

`add` declares `: number`, and `greet` declares `: string`, making the return contracts explicit without changing either body.

```vue annotate="add:2,6"
<script setup lang="ts">
const add = (a: number, b: number): number => {
  return a + b
}

function greet(name: string): string {
  return `Hello, ${name}`
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L49) · [All rules](all.md)

### `script/require-prop-type-constructor`

Require prop `type` values to be constructors rather than string literals

[Bad](#script-require-prop-type-constructor-bad) · [Good](#script-require-prop-type-constructor-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-prop-type-constructor": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-type-constructor-bad"></span>

**Bad**

The prop declarations use strings `"String"` and `"Number"` as runtime types, including inside the constructor array. Those strings are not constructor functions.

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
  props: {
    // The type should be the `String` constructor, not the string "String".
    name: "String",
    age: { type: "Number" },
    id: { type: ["String", "Number"] }
  }
}
</script>
```

<span id="script-require-prop-type-constructor-good"></span>

**Good**

The declarations use the actual `String` and `Number` identifiers, including the union array `[String, Number]`.

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
  props: {
    name: String,
    age: { type: Number },
    id: { type: [String, Number] }
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L59) · [All rules](all.md)

### `script/require-prop-types`

Require every prop to declare a type

[Bad](#script-require-prop-types-bad) · [Good](#script-require-prop-types-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-prop-types": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-types-bad"></span>

**Bad**

The array entry declares only the name `status`; the `null` value and empty descriptor declare no runtime prop type either.

```vue annotate="remove:3,4,5,6,8,9"
<script lang="ts">
export default {
  props: ['status']            // array form: no types
}

export default {
  props: {
    status: null,              // no type
    other: {}                  // empty descriptor: no type
  }
}
</script>
```

<span id="script-require-prop-types-good"></span>

**Good**

`status: String` supplies a shorthand constructor, and `other` supplies `type: Number` inside its descriptor. Both props now carry type declarations.

```vue annotate="add:4,5"
<script lang="ts">
export default {
  props: {
    status: String,
    other: { type: Number, default: 0 }
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58) · [All rules](all.md)

### `script/require-symbol-provide`

Recommend using Symbol as injection key for provide/inject

[Bad](#script-require-symbol-provide-bad) · [Good](#script-require-symbol-provide-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-symbol-provide": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-symbol-provide-bad"></span>

**Bad**

`provide` and `inject` use literal string keys such as `'user'` and `'theme'`, which can collide with another provider using the same spelling.

```vue annotate="remove:1,2,3,4,6,7"
<script setup lang="ts">
// String keys can collide
provide('user', user)
const user = inject('user')

// Magic strings are error-prone
provide('theme', { dark: true })
</script>
```

<span id="script-require-symbol-provide-good"></span>

**Good**

The shared `UserKey` is created with `Symbol` and annotated as `InjectionKey<User>`; both calls pass that key instead of a literal string.

```vue annotate="add:1,2,3,5,6,7,8,9"
<script lang="ts">
// Define injection key with Symbol
export const UserKey: InjectionKey<User> = Symbol('user')

// Provide with Symbol
provide(UserKey, user)

// Inject with Symbol
const user = inject(UserKey)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) · [All rules](all.md)

### `script/require-typed-object-prop`

Require an explicit type on a prop whose runtime type is `Object` or `Array`

[Bad](#script-require-typed-object-prop-bad) · [Good](#script-require-typed-object-prop-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-object-prop": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-object-prop-bad"></span>

**Bad**

Bare `Object` and `Array` constructors describe only broad runtime categories, so neither `user` nor the `items` element shape has an explicit static type.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

<span id="script-require-typed-object-prop-good"></span>

**Good**

`PropType<User>` and `PropType<User[]>` add the object and element types while retaining the same runtime constructors.

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) · [All rules](all.md)

### `script/require-typed-ref`

Require an explicit type argument on a ref() initialized with no value, null, or undefined

[Bad](#script-require-typed-ref-bad) · [Good](#script-require-typed-ref-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-ref-bad"></span>

**Bad**

The imported `ref` calls have neither a type argument nor a useful initial value: no argument, `null`, and `undefined` cannot infer the intended future value type.

```vue annotate="remove:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref()           // Ref<undefined>
const b = ref(null)       // Ref<null>
const c = ref(undefined)  // Ref<undefined>
</script>
```

<span id="script-require-typed-ref-good"></span>

**Good**

Explicit type arguments describe the string and nullable User refs. `ref(0)` already has a concrete numeric initializer and can rely on inference.

```vue annotate="add:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref<string>()
const b = ref<User | null>(null)
const c = ref(0)          // inferred Ref<number>
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55) · [All rules](all.md)

### `script/require-valid-default-prop`

Require a prop's default value to be valid for its declared type

[Bad](#script-require-valid-default-prop-bad) · [Good](#script-require-valid-default-prop-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-valid-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-valid-default-prop-bad"></span>

**Bad**

The Number and Boolean props receive mismatched scalar defaults, and the Array and Object props use shared literal values instead of factories.

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
  props: {
    count: { type: Number, default: '0' },     // string default for Number
    enabled: { type: Boolean, default: 1 },     // non-boolean default for Boolean
    items: { type: Array, default: [] },        // literal must be a factory
    config: { type: Object, default: {} }       // literal must be a factory
  }
}
</script>
```

<span id="script-require-valid-default-prop-good"></span>

**Good**

The scalar defaults become `0` and `false`; the array and object defaults become functions returning fresh values. The `[String, Number]` example accepts its string default because it matches one declared type.

```vue annotate="add:4,5,6,7,8"
<script lang="ts">
export default {
  props: {
    count: { type: Number, default: 0 },
    enabled: { type: Boolean, default: false },
    items: { type: Array, default: () => [] },
    config: { type: Object, default: () => ({}) },
    label: { type: [String, Number], default: '' }
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) · [All rules](all.md)

### `script/return-in-computed-property`

Require a return value in every computed getter

[Bad](#script-return-in-computed-property-bad) · [Good](#script-return-in-computed-property-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-computed-property": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-computed-property-bad"></span>

**Bad**

The block-bodied computed getter evaluates `1 + 2` but never returns it, leaving the computed value undefined.

```vue annotate="remove:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { 1 + 2; });
</script>
```

<span id="script-return-in-computed-property-good"></span>

**Good**

`return 1 + 2` turns the expression into the getter’s returned value. The rule looks for a value-returning return in the getter itself, not merely an expression statement.

```vue annotate="add:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { return 1 + 2; });
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) · [All rules](all.md)

### `script/return-in-emits-validator`

Require a return value in every Options API emits validator

[Bad](#script-return-in-emits-validator-bad) · [Good](#script-return-in-emits-validator-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

Use a block-body arrow for the currently supported SFC filter. The underlying validator also handles method shorthand, but the current SFC prefilter does not reliably dispatch that shape.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-emits-validator": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-emits-validator-bad"></span>

**Bad**

The `submit` validator logs the payload but does not return a validation result, so its block body yields undefined.

```vue annotate="remove:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

<span id="script-return-in-emits-validator-good"></span>

**Good**

`return payload != null` supplies a boolean validation result for the submitted payload instead of ending without a returned value.

```vue annotate="add:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L59) · [All rules](all.md)

### `script/valid-define-emits`

Enforce valid defineEmits() usage (no type+runtime args, no local references, single call)

[Bad](#script-valid-define-emits-bad) · [Good](#script-valid-define-emits-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-emits": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-emits-bad"></span>

**Bad**

The same `defineEmits` call supplies both a type argument and the runtime array `["save"]`, mixing two mutually exclusive declarations.

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

<span id="script-valid-define-emits-good"></span>

**Good**

Removing the runtime argument leaves a single type-based event declaration for `save`.

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) · [All rules](all.md)

### `script/valid-define-options`

Enforce valid defineOptions() usage (single object arg, no props/emits/expose/slots)

[Bad](#script-valid-define-options-bad) · [Good](#script-valid-define-options-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-options": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-options-bad"></span>

**Bad**

The first call puts the dedicated `props` declaration inside `defineOptions`; the later calls also repeat the macro and include a non-object argument. These illustrate the forbidden shape and repeated-call constraints.

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
defineOptions({ props: ['foo'] })   // use defineProps instead
defineOptions({ name: 'Foo' })
defineOptions({ name: 'Bar' })      // duplicate call
defineOptions('Foo')                // not an object literal
</script>
```

<span id="script-valid-define-options-good"></span>

**Good**

One `defineOptions` call receives an object containing only the supported ordinary options `name` and `inheritAttrs`.

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: 'Foo', inheritAttrs: false })
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) · [All rules](all.md)

### `script/valid-define-props`

Enforce valid defineProps() usage (single call, not both type and runtime args, no local references)

[Bad](#script-valid-define-props-bad) · [Good](#script-valid-define-props-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-props-bad"></span>

**Bad**

The same `defineProps` call supplies both `{ title: string }` as a type argument and `{ title: String }` as a runtime argument, which the compiler does not permit together.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

<span id="script-valid-define-props-good"></span>

**Good**

Removing the runtime object leaves one type-based declaration for `title` instead of combining both declaration forms.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) · [All rules](all.md)

### `script/valid-next-tick`

Require the result of a nextTick() call to be awaited, chained, or given a callback

[Bad](#script-valid-next-tick-bad) · [Good](#script-valid-next-tick-good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-next-tick": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-next-tick-bad"></span>

**Bad**

The imported `nextTick()` is a bare expression with no callback, so its returned Promise is ignored and no work waits for the DOM flush.

```vue annotate="remove:3"
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

<span id="script-valid-next-tick-good"></span>

**Good**

`await nextTick()` consumes the Promise and explicitly waits for the next DOM update before subsequent setup code continues.

```vue annotate="add:3"
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) · [All rules](all.md)

### `type/no-floating-promises`

Disallow floating (unhandled) Promises

[Bad](#type-no-floating-promises-bad) · [Good](#type-no-floating-promises-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: No rule-specific options. Severity and preset selection are configurable.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-floating-promises": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-floating-promises-bad"></span>

**Bad**

The async `save` function returns a Promise, but the standalone `save()` call neither awaits nor returns it and does not explicitly mark intentional disposal.

```vue annotate="remove:3"
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

<span id="type-no-floating-promises-good"></span>

**Good**

`void save()` explicitly marks the fire-and-forget intent accepted by this rule. This is an explicit disposal marker, not a rejection handler.

```vue annotate="add:3"
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) · [All rules](all.md)

### `type/no-reactivity-loss`

Disallow plain snapshots of reactive values across assignments and calls

[Bad](#type-no-reactivity-loss-bad) · [Good](#type-no-reactivity-loss-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: No rule-specific options. Severity and preset selection are configurable.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-reactivity-loss": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-reactivity-loss-bad"></span>

**Bad**

`const count = state.count` takes a plain numeric snapshot of the reactive property, so later updates of `state.count` are not reflected in that binding.

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0 });
const count = state.count;
</script>
```

<span id="type-no-reactivity-loss-good"></span>

**Good**

`toRef(state, "count")` keeps `count` linked to the original reactive property rather than copying its current primitive value.

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRef } from "vue";
const state = reactive({ count: 0 });
const count = toRef(state, "count");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) · [All rules](all.md)

### `type/no-unsafe-template-binding`

Disallow template bindings that resolve to unsafe types

[Bad](#type-no-unsafe-template-binding-bad) · [Good](#type-no-unsafe-template-binding-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: No rule-specific options. Severity and preset selection are configurable.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-unsafe-template-binding": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-unsafe-template-binding-bad"></span>

**Bad**

The interpolated `value` is explicitly typed as `any`, so the checker cannot give the template binding a safe concrete type.

```vue annotate="remove:2"
<script setup lang="ts">
const value: any = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

<span id="type-no-unsafe-template-binding-good"></span>

**Good**

Changing the annotation to `string` gives the same interpolation a concrete, checkable type without changing the rendered value.

```vue annotate="add:2"
<script setup lang="ts">
const value: string = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) · [All rules](all.md)

### `type/require-typed-emits`

Require type definition for defineEmits

[Bad](#type-require-typed-emits-bad) · [Good](#type-require-typed-emits-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: No rule-specific options. Severity and preset selection are configurable.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-emits": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-emits-bad"></span>

**Bad**

The array-only `defineEmits(["save"])` declares the event name without a typed payload contract.

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

<span id="type-require-typed-emits-good"></span>

**Good**

`defineEmits<{ save: [] }>()` declares the typed `save` event with an empty payload tuple, explicitly stating that it takes no payload arguments.

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) · [All rules](all.md)

### `type/require-typed-props`

Require type definition for defineProps

[Bad](#type-require-typed-props-bad) · [Good](#type-require-typed-props-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: No rule-specific options. Severity and preset selection are configurable.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-props": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-props-bad"></span>

**Bad**

The array-only `defineProps(["title"])` declares `title` by name without giving it a type.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps(["title"]);
</script>
```

<span id="type-require-typed-props-good"></span>

**Good**

`defineProps<{ title: string }>()` gives `title` an explicit string type instead of a name-only runtime declaration.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57) · [All rules](all.md)

### `type/strict-boolean-expressions`

Require safe boolean expressions in script and template conditions

[Bad](#type-strict-boolean-expressions-bad) · [Good](#type-strict-boolean-expressions-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: See [typed options and defaults](options.md).

Enable typeAware and this rule explicitly. The default disallows nullable numbers, while non-null numbers are allowed.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/strict-boolean-expressions": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-strict-boolean-expressions-bad"></span>

**Bad**

`if (count)` relies on the truthiness of a nullable numeric binding instead of an explicit boolean test; it also conflates zero with absence.

```vue annotate="remove:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

<span id="type-strict-boolean-expressions-good"></span>

**Good**

`count !== undefined && count > 0` separately tests presence and positivity, producing an explicit boolean condition after narrowing the optional value.

```vue annotate="add:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) · [All rules](all.md)
