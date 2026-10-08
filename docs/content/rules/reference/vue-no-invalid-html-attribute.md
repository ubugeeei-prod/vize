---
title: "vue/no-invalid-html-attribute"
---

# `vue/no-invalid-html-attribute`

Disallow invalid static values for HTML attributes

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
        "vue/no-invalid-html-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The anchor uses stylesheet as a rel value, although that value belongs to stylesheet link elements.

```vue
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

## Good

The anchor uses help, a rel value appropriate for a linked help resource.

```vue
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) · [All rules](../all.md)
