---
title: "vue/component-definition-name-casing"
description: "Enforce PascalCase or kebab-case for component definition names."
---

# `vue/component-definition-name-casing`

Enforce PascalCase or kebab-case for component definition names.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

The component filename is checked. PascalCase and kebab-case are accepted; mixed casing is reported.

## Bad

The filename myComponent.vue mixes a lowercase initial with an internal uppercase letter instead of using PascalCase or kebab-case.

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

## Good

Renaming the file to MyComponent.vue uses PascalCase; its template content is unchanged.

`MyComponent.vue`

```vue
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
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36)
