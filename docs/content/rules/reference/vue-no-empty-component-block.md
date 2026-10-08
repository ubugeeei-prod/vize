---
title: "vue/no-empty-component-block"
---

# `vue/no-empty-component-block`

Disallow empty SFC blocks

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
        "vue/no-empty-component-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The template, script, and style blocks contain no meaningful content.

```vue
<template></template>

<script></script>

<style>
</style>
```

## Good

Each retained block contains actual markup, script declarations, or style declarations.

```vue
<template>
<div>Hello</div>
</template>

<script setup>
const message = "Hello";
</script>

<style scoped>
.button { color: red; }
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42) · [All rules](../all.md)
