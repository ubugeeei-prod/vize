---
title: "vue/valid-v-show"
---

# `vue/valid-v-show`

Enforce valid `v-show` directives

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
        "vue/valid-v-show": "error"
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
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

## Good

```vue
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) · [All rules](../all.md)
