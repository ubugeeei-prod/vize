---
title: "vue/no-duplicate-attributes"
---

# `vue/no-duplicate-attributes`

Disallow duplicate attributes on the same element

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
        "vue/no-duplicate-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The same button declares class twice instead of one combined class value.

```vue
<template>
  <button class="primary" class="large">Save</button>
</template>
```

## Good

Both class tokens appear in a single class attribute.

```vue
<template>
  <button class="primary large">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) · [All rules](../all.md)
