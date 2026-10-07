---
title: "vue/component-name-in-template-casing"
---

# `vue/component-name-in-template-casing`

Enforce specific casing for component names in templates

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](../options.md).

## Configuration (Vite+)

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

## Bad

```vue
<template>
  <my-component />
  <myComponent />
</template>
```

## Good

```vue
<template>
  <MyComponent />
  <RouterView />
  <slot />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L30) · [All rules](../all.md)
