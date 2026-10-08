---
title: "vue/component-definition-name-casing"
---

# `vue/component-definition-name-casing`

Enforce PascalCase or kebab-case for component definition names

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

The component filename is checked. PascalCase and kebab-case are accepted; mixed casing is reported.

## Configuration (Vite+)

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

## Bad

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

## Good

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) · [All rules](../all.md)
