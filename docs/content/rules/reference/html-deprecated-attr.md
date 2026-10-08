---
title: "html/deprecated-attr"
---

# `html/deprecated-attr`

Disallow deprecated HTML attributes

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "html/deprecated-attr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The paragraph uses the deprecated presentational `align` attribute.

```vue
<template>
<p align="center">Notice</p>
</template>
```

## Good

The class and `text-align: center` declaration express the alignment through CSS.

```vue
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) · [All rules](../all.md)
