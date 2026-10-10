---
title: "Vue Rules: Components and Props"
---

# Vue Rules: Components and Props

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [Bad](#vue-component-definition-name-casing-bad) · [Good](#vue-component-definition-name-casing-good) | Enforce PascalCase or kebab-case for component definition names |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [Bad](#vue-component-name-in-template-casing-bad) · [Good](#vue-component-name-in-template-casing-good) | Enforce specific casing for component names in templates |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [Bad](#vue-multi-word-component-names-bad) · [Good](#vue-multi-word-component-names-good) | Require component names to be multi-word |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [Bad](#vue-no-mutating-props-bad) · [Good](#vue-no-mutating-props-good) | Disallow mutating component props |
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [Bad](#vue-no-non-component-keep-alive-child-bad) · [Good](#vue-no-non-component-keep-alive-child-good) | Disallow plain element wrappers directly below `<KeepAlive>` |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [Bad](#vue-no-reserved-component-names-bad) · [Good](#vue-no-reserved-component-names-good) | Disallow the use of reserved names as component names |
| [`vue/no-unused-components`](#vue-no-unused-components) | [Bad](#vue-no-unused-components-bad) · [Good](#vue-no-unused-components-good) | Disallow registering components that are not used inside templates |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [Bad](#vue-no-unused-properties-bad) · [Good](#vue-no-unused-properties-good) | Disallow unused properties defined in defineProps |
| [`vue/require-component-is`](#vue-require-component-is) | [Bad](#vue-require-component-is-bad) · [Good](#vue-require-component-is-good) | Require `v-bind:is` on `<component>` elements |
| [`vue/require-component-registration`](#vue-require-component-registration) | [Bad](#vue-require-component-registration-bad) · [Good](#vue-require-component-registration-good) | Require explicit import or registration for components |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/component-definition-name-casing`

Enforce PascalCase or kebab-case for component definition names

[Bad](#vue-component-definition-name-casing-bad) · [Good](#vue-component-definition-name-casing-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

The component filename is checked. PascalCase and kebab-case are accepted; mixed casing is reported.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/component-definition-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-component-definition-name-casing-bad"></span>

**Bad**

The filename myComponent.vue mixes a lowercase initial with an internal uppercase letter instead of using PascalCase or kebab-case.

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

<span id="vue-component-definition-name-casing-good"></span>

**Good**

Renaming the file to MyComponent.vue uses PascalCase; its template content is unchanged.

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) · [All rules](all.md)

### `vue/component-name-in-template-casing`

Enforce specific casing for component names in templates

[Bad](#vue-component-name-in-template-casing-bad) · [Good](#vue-component-name-in-template-casing-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](options.md).

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/component-name-in-template-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-component-name-in-template-casing-bad"></span>

**Bad**

The component is written in kebab-case and camelCase under the PascalCase convention.

```vue annotate="remove:5,6"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <my-component />
  <myComponent />
</template>
```

<span id="vue-component-name-in-template-casing-good"></span>

**Good**

MyComponent uses PascalCase; native slot syntax remains lowercase.

```vue annotate="add:5,6,7"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <MyComponent />
  <RouterView />
  <slot />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31) · [All rules](all.md)

### `vue/multi-word-component-names`

Require component names to be multi-word

[Bad](#vue-multi-word-component-names-bad) · [Good](#vue-multi-word-component-names-good)

Default severity: `error`  
Presets: `essential`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

The filename is the finding. Rename the same component; changing a child tag does not fix it.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/multi-word-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-multi-word-component-names-bad"></span>

**Bad**

Item.vue gives the component a single-word name.

`Item.vue`

```vue
<template><p>Item</p></template>
```

<span id="vue-multi-word-component-names-good"></span>

**Good**

TodoItem.vue gives the same template a multi-word component name.

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) · [All rules](all.md)

### `vue/no-mutating-props`

Disallow mutating component props

[Bad](#vue-no-mutating-props-bad) · [Good](#vue-no-mutating-props-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](options.md).

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-mutating-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-mutating-props-bad"></span>

**Bad**

Incrementing props.count writes directly to a value supplied by the parent.

```vue annotate="remove:4"
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

<span id="vue-no-mutating-props-good"></span>

**Good**

The component emits update:count with the next value, leaving the parent responsible for updating the prop.

```vue annotate="add:3,5,6,7"
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L42) · [All rules](all.md)

### `vue/no-non-component-keep-alive-child`

Disallow plain element wrappers directly below `<KeepAlive>`

[Bad](#vue-no-non-component-keep-alive-child-bad) · [Good](#vue-no-non-component-keep-alive-child-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-non-component-keep-alive-child": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-non-component-keep-alive-child-bad"></span>

**Bad**

KeepAlive conditionally wraps a native div rather than directly caching UserCard.

```vue annotate="remove:3"
<template>
  <KeepAlive>
    <div v-if="ready">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

<span id="vue-no-non-component-keep-alive-child-good"></span>

**Good**

The first example makes UserCard the conditional child. The v-show wrapper illustrates a shape outside this conditional-child check, not a promise that the native wrapper is cached.

```vue annotate="add:3,4,5,6"
<template>
  <KeepAlive>
    <UserCard v-if="ready" />
  </KeepAlive>
  <KeepAlive>
    <div v-show="opened">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L14) · [All rules](all.md)

### `vue/no-reserved-component-names`

Disallow the use of reserved names as component names

[Bad](#vue-no-reserved-component-names-bad) · [Good](#vue-no-reserved-component-names-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-reserved-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-reserved-component-names-bad"></span>

**Bad**

The component name button conflicts with a native HTML element name.

```vue annotate="remove:1,2,3,4"
<script>
export default {
  name: "button",
};
</script>
```

<span id="vue-no-reserved-component-names-good"></span>

**Good**

AppButton is an application component name and does not reuse the native button name.

```vue annotate="add:1,2,4,5,6,7,8,9"
<script setup lang="ts">
defineOptions({ name: "AppButton" });
</script>

<template>
  <Transition>
    <AppButton />
  </Transition>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) · [All rules](all.md)

### `vue/no-unused-components`

Disallow registering components that are not used inside templates

[Bad](#vue-no-unused-components-bad) · [Good](#vue-no-unused-components-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-components-bad"></span>

**Bad**

UserAvatar is imported as a component but the template never renders it.

```vue annotate="remove:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <p>{{ user.name }}</p>
</template>
```

<span id="vue-no-unused-components-good"></span>

**Good**

The template renders the imported UserAvatar and passes the user binding.

```vue annotate="add:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L46) · [All rules](all.md)

### `vue/no-unused-properties`

Disallow unused properties defined in defineProps

[Bad](#vue-no-unused-properties-bad) · [Good](#vue-no-unused-properties-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-properties-bad"></span>

**Bad**

The component declares description as a prop but renders only title.

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
```

<span id="vue-no-unused-properties-good"></span>

**Good**

Both declared props are referenced by the template.

```vue annotate="add:7"
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ description }}</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) · [All rules](all.md)

### `vue/require-component-is`

Require `v-bind:is` on `<component>` elements

[Bad](#vue-require-component-is-bad) · [Good](#vue-require-component-is-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-component-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-component-is-bad"></span>

**Bad**

The dynamic `<component>` has no `is` target, so Vue cannot choose a component to render.

```vue annotate="remove:2"
<template>
  <component />
</template>
```

<span id="vue-require-component-is-good"></span>

**Good**

`:is="currentComponent"` supplies the component selection; the binding may change at runtime.

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) · [All rules](all.md)

### `vue/require-component-registration`

Require explicit import or registration for components

[Bad](#vue-require-component-registration-bad) · [Good](#vue-require-component-registration-good)

Default severity: `warning`  
Presets: `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](options.md).

List explicit component names supplied by application plugins or Musea previewSetup. PascalCase and kebab-case spellings are accepted; regular expressions are not interpreted. Options do not enable the rule. Later layers replace the list; an empty list clears inherited names.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-component-registration": "warn"
      },
      "ruleOptions": {
        "vue/require-component-registration": {
          "globals": [
            "MyButton",
            "MyIcon"
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

<span id="vue-require-component-registration-bad"></span>

**Bad**

`MissingWidget` is neither registered nor included in the configured global-component allowlist.

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

<span id="vue-require-component-registration-good"></span>

**Good**

`MyButton` is listed in the example's `globals` option. That option exempts a known global component; it does not register or import it.

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [All rules](all.md)
