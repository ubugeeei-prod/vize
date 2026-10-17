---
title: "vue/valid-v-else-if"
---

# `vue/valid-v-else-if`

Enforce valid `v-else-if` directives

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
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-else-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The examples leave `v-else-if` without a preceding conditional branch or a condition, or give it an argument or modifier.

```vue
<template>
<div v-else-if="foo"></div> <!-- without preceding v-if -->
<div v-if="foo"></div><div v-else-if></div>
<div v-if="foo"></div><div v-else-if:arg="bar"></div>
<div v-if="foo"></div><div v-else-if.mod="bar"></div>
</template>
```

## Good

Place `v-else-if` with a condition immediately after the `v-if` or `v-else-if` branch it continues.

```vue
<template>
<div v-if="foo"></div>
<div v-else-if="bar"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else_if.rs#L34) · [All rules](../all.md)
