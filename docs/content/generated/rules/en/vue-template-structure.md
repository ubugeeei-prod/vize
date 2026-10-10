---
title: "Vue Rules: Template Structure"
---

# Vue Rules: Template Structure

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/no-child-content`](#vue-no-child-content) | [Bad](#vue-no-child-content-bad) · [Good](#vue-no-child-content-good) | Disallow child content when using v-html or v-text |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [Bad](#vue-no-dupe-v-else-if-bad) · [Good](#vue-no-dupe-v-else-if-good) | Disallow duplicate conditions in `v-if` / `v-else-if` chains |
| [`vue/no-lone-template`](#vue-no-lone-template) | [Bad](#vue-no-lone-template-bad) · [Good](#vue-no-lone-template-good) | Disallow unnecessary `<template>` elements |
| [`vue/no-template-key`](#vue-no-template-key) | [Bad](#vue-no-template-key-bad) · [Good](#vue-no-template-key-good) | Disallow `key` attribute on `<template>` |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [Bad](#vue-no-template-shadow-bad) · [Good](#vue-no-template-shadow-good) | Disallow variable names that shadow variables in outer scope |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [Bad](#vue-no-unused-vars-bad) · [Good](#vue-no-unused-vars-good) | Disallow unused variable definitions in v-for and v-slot directives |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [Bad](#vue-no-use-v-if-with-v-for-bad) · [Good](#vue-no-use-v-if-with-v-for-good) | Disallow using `v-if` on the same element as `v-for` |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [Bad](#vue-no-useless-template-attributes-bad) · [Good](#vue-no-useless-template-attributes-good) | Disallow useless attributes on `<template>` elements |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [Bad](#vue-require-v-for-key-bad) · [Good](#vue-require-v-for-key-good) | Require `v-bind:key` with `v-for` directives |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/no-child-content`

Disallow child content when using v-html or v-text

[Bad](#vue-no-child-content-bad) · [Good](#vue-no-child-content-good)

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
        "vue/no-child-content": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-child-content-bad"></span>

**Bad**

v-text replaces the paragraph content, so the authored fallback text cannot survive that directive.

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

<span id="vue-no-child-content-good"></span>

**Good**

Removing the child text leaves v-text as the single source of paragraph content.

```vue annotate="add:2"
<template>
  <p v-text="message" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [All rules](all.md)

### `vue/no-dupe-v-else-if`

Disallow duplicate conditions in `v-if` / `v-else-if` chains

[Bad](#vue-no-dupe-v-else-if-bad) · [Good](#vue-no-dupe-v-else-if-good)

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
        "vue/no-dupe-v-else-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-dupe-v-else-if-bad"></span>

**Bad**

The else-if repeats the ready condition already tested by the first branch, making that later branch unreachable.

```vue annotate="remove:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

<span id="vue-no-dupe-v-else-if-good"></span>

**Good**

The second branch tests loading, a distinct state that can reach the else-if.

```vue annotate="add:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) · [All rules](all.md)

### `vue/no-lone-template`

Disallow unnecessary `<template>` elements

[Bad](#vue-no-lone-template-bad) · [Good](#vue-no-lone-template-good)

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
        "vue/no-lone-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-lone-template-bad"></span>

**Bad**

The inner template has no directive or slot role that gives it a structural purpose.

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

<span id="vue-no-lone-template-good"></span>

**Good**

Removing the unnecessary wrapper leaves the paragraph directly inside div.

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) · [All rules](all.md)

### `vue/no-template-key`

Disallow `key` attribute on `<template>`

[Bad](#vue-no-template-key-bad) · [Good](#vue-no-template-key-good)

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
        "vue/no-template-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-key-bad"></span>

**Bad**

A non-loop template wrapper has a key even though it is not the keyed iteration boundary.

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

<span id="vue-no-template-key-good"></span>

**Good**

The key belongs to a template v-for iteration, where it identifies each repeated fragment.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) · [All rules](all.md)

### `vue/no-template-shadow`

Disallow variable names that shadow variables in outer scope

[Bad](#vue-no-template-shadow-bad) · [Good](#vue-no-template-shadow-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

The current check compares nested v-for bindings. It does not report a single v-for binding merely because it shares a script binding's name.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-shadow": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-shadow-bad"></span>

**Bad**

The inner v-for declares item again and hides the outer item binding inside the nested loop.

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

<span id="vue-no-template-shadow-good"></span>

**Good**

The inner loop declares child, leaving item available for the outer row and child for the nested row.

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [All rules](all.md)

### `vue/no-unused-vars`

Disallow unused variable definitions in v-for and v-slot directives

[Bad](#vue-no-unused-vars-bad) · [Good](#vue-no-unused-vars-good)

Default severity: `warning`  
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
        "vue/no-unused-vars": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-vars-bad"></span>

**Bad**

The loop declares an unused index and the slot declares foo without referencing it.

```vue annotate="remove:2,3,4"
<template>
  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ foo }">
    <span>Hello</span>
  </template>
</template>
```

<span id="vue-no-unused-vars-good"></span>

**Good**

The examples consume index or mark it intentionally unused as _index, and the slot renders data. Index keys are only a usage example here, not a recommendation for stable item identity.

```vue annotate="add:2,3,4,5"
<template>
  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
  <li v-for="(item, _index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ data }">
    <span>{{ data }}</span>
  </template>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) · [All rules](all.md)

### `vue/no-use-v-if-with-v-for`

Disallow using `v-if` on the same element as `v-for`

[Bad](#vue-no-use-v-if-with-v-for-bad) · [Good](#vue-no-use-v-if-with-v-for-good)

Default severity: `warning`  
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
        "vue/no-use-v-if-with-v-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-use-v-if-with-v-for-bad"></span>

**Bad**

The same list element combines v-if and v-for and tests visibility through the loop binding.

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

<span id="vue-no-use-v-if-with-v-for-good"></span>

**Good**

A computed collection filters the visible items before the template iterates over them.

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const visibleItems = computed(() => items.filter((item) => item.visible));
</script>

<template>
  <li v-for="item in visibleItems" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) · [All rules](all.md)

### `vue/no-useless-template-attributes`

Disallow useless attributes on `<template>` elements

[Bad](#vue-no-useless-template-attributes-bad) · [Good](#vue-no-useless-template-attributes-good)

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
        "vue/no-useless-template-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-template-attributes-bad"></span>

**Bad**

The conditional template has a class, but this structural wrapper does not render a DOM element to receive it.

```vue annotate="remove:2"
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

<span id="vue-no-useless-template-attributes-good"></span>

**Good**

The class moves to the paragraph that actually renders while v-if stays on the structural template.

```vue annotate="add:2"
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [All rules](all.md)

### `vue/require-v-for-key`

Require `v-bind:key` with `v-for` directives

[Bad](#vue-require-v-for-key-bad) · [Good](#vue-require-v-for-key-good)

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
        "vue/require-v-for-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-v-for-key-bad"></span>

**Bad**

Each repeated `<li>` lacks a key that identifies its corresponding item during list updates.

```vue annotate="remove:2"
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

<span id="vue-require-v-for-key-good"></span>

**Good**

`:key="item.id"` gives each repeated node the item's identity rather than its current position.

```vue annotate="add:2"
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L35) · [All rules](all.md)
