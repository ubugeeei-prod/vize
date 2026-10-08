---
title: "a11y/heading-levels"
---

# `a11y/heading-levels`

Disallow skipping heading levels

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
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
        "a11y/heading-levels": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The heading sequence jumps directly from `h1` to `h3`, skipping level two.

```vue
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

## Good

Changing the billing heading to `h2` preserves a consecutive heading hierarchy.

```vue
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) · [All rules](../all.md)
