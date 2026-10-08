---
title: "vue/valid-v-memo"
---

# `vue/valid-v-memo`

Enforce valid `v-memo` directives

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
        "vue/valid-v-memo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

Bare `v-memo` gives Vue no dependency expression for deciding when to reuse the subtree.

```vue
<template>
  <div v-memo></div>
</template>
```

## Good

`v-memo="[valueA, valueB]"` supplies the dependency array used for memoization.

```vue
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) · [All rules](../all.md)
