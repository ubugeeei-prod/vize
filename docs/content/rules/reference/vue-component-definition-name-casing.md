---
title: "vue/component-definition-name-casing"
description: "Enforce PascalCase or kebab-case for component definition names."
---

# `vue/component-definition-name-casing`

Enforce PascalCase or kebab-case for component definition names.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

Checks the static name of a component definition: the name option, defineOptions({ name }) and the first argument of app.component(). The file name is vue/component-file-name-casing.

## Bad

The name my_component declared through defineOptions separates its words with an underscore instead of using PascalCase.

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "my_component" });
</script>
<template><p>Content</p></template>
```

## Good

The declared name MyComponent uses PascalCase, the default casing.

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "MyComponent" });
</script>
<template><p>Content</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/component-definition-name-casing": "warn"
    },
    "ruleOptions": {
      "vue/component-definition-name-casing": "PascalCase"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L68)
