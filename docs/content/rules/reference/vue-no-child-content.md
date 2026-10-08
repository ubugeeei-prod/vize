---
title: "vue/no-child-content"
---

# `vue/no-child-content`

Disallow child content when using v-html or v-text

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
        "vue/no-child-content": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

v-text replaces the paragraph content, so the authored fallback text cannot survive that directive.

```vue
<template>
  <p v-text="message">Fallback text</p>
</template>
```

## Good

Removing the child text leaves v-text as the single source of paragraph content.

```vue
<template>
  <p v-text="message" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [All rules](../all.md)
