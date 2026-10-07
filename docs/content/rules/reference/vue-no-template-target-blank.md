---
title: "vue/no-template-target-blank"
---

# `vue/no-template-target-blank`

Disallow target="_blank" without rel="noopener noreferrer"

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
        "vue/no-template-target-blank": "warn"
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
<a href="https://example.com" target="_blank">x</a>
</template>
```

## Good

```vue
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) · [All rules](../all.md)
