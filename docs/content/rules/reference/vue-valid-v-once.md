---
title: "vue/valid-v-once"
---

# `vue/valid-v-once`

Enforce valid `v-once` directives

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
        "vue/valid-v-once": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`v-once` has a value, argument, or modifier, although this directive is a value-free render-once marker.

```vue
<template>
<div v-once="foo"></div>
<div v-once:arg></div>
<div v-once.mod></div>
</template>
```

## Good

Bare `v-once` marks the subtree for one-time rendering without unsupported syntax.

```vue
<template>
<div v-once></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) · [All rules](../all.md)
