---
title: "vue/no-boolean-attr-value"
---

# `vue/no-boolean-attr-value`

Disallow explicit values for boolean HTML attributes

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
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
        "vue/no-boolean-attr-value": "warn"
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
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

## Good

```vue
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) · [All rules](../all.md)
