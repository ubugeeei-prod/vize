---
title: "vue/no-textarea-mustache"
---

# `vue/no-textarea-mustache`

Disallow mustache interpolation in `<textarea>`

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
        "vue/no-textarea-mustache": "error"
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
  <textarea>{{ message }}</textarea>
</template>
```

## Good

```vue
<template>
  <textarea v-model="message"></textarea>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [All rules](../all.md)
