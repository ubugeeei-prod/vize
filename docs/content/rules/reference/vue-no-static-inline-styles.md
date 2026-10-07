---
title: "vue/no-static-inline-styles"
---

# `vue/no-static-inline-styles`

Disallow static inline style attributes

Default severity: `warning`  
Presets: _none_  
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
        "vue/no-static-inline-styles": "warn"
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
<p style="color: red">Notice</p>
</template>
```

## Good

```vue
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) · [All rules](../all.md)
