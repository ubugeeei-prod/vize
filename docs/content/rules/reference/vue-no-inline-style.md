---
title: "vue/no-inline-style"
---

# `vue/no-inline-style`

Discourage use of inline style attributes

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
        "vue/no-inline-style": "warn"
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
  <div style="color: red">Text</div>
</template>
```

## Good

```vue
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) · [All rules](../all.md)
