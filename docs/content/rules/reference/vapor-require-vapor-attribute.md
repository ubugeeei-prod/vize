---
title: "vapor/require-vapor-attribute"
---

# `vapor/require-vapor-attribute`

Suggest adding vapor attribute to script setup

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Not implemented for SFC lint  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Current support: `no-sfc-finding`

This rule is a placeholder with an empty callback. Adding vapor selects Vapor compilation; the current linter does not report this catalog ID for its absence.

## Configured ID (currently no SFC finding)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The script setup block lacks the Vapor compilation attribute. This is an intended convention: the current empty rule callback does not diagnose it.

```vue
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

## Good

Adding vapor selects Vapor compilation. It demonstrates the intended repair and does not imply that the current linter emits this catalog rule.

```vue
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) · [All rules](../all.md)
