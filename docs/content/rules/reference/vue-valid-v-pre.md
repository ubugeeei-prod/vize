---
title: "vue/valid-v-pre"
---

# `vue/valid-v-pre`

Enforce valid `v-pre` directives

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
        "vue/valid-v-pre": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`v-pre` has an argument, modifier, or value, although this directive is a value-free marker that skips compilation.

```vue
<template>
<div v-pre:arg></div>
<div v-pre.mod></div>
<div v-pre="foo"></div>
</template>
```

## Good

Bare `v-pre` leaves the subtree uncompiled, so its mustaches render as written.

```vue
<template>
<div v-pre>{{ raw }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_pre.rs#L30) · [All rules](../all.md)
