---
title: "vue/valid-template-root"
---

# `vue/valid-template-root`

Enforce a valid `<template>` root for Vue 3 fragment semantics

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-template-root": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

A plain nested `<template>` occupies the template root without a directive that gives it a rendering role.

```vue
<template>
  <template>content</template>
</template>
```

## Good

The `<div>` is a renderable root element. This example does not impose a universal single-root restriction on Vue 3 fragments.

```vue
<template>
  <div>content</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) · [All rules](../all.md)
