---
title: "vue/valid-v-text"
---

# `vue/valid-v-text`

Enforce valid `v-text` directives

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
        "vue/valid-v-text": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`v-text` lacks its text expression or uses an unsupported argument/modifier.

```vue
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

## Good

`v-text="msg"` is syntactically valid. The separate `vue/no-v-text` style rule can still prefer interpolation.

```vue
<template>
<div v-text="msg"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) · [All rules](../all.md)
