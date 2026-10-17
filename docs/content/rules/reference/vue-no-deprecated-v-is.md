---
title: "vue/no-deprecated-v-is"
---

# `vue/no-deprecated-v-is`

Disallow the deprecated `v-is` directive

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
        "vue/no-deprecated-v-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`v-is` was deprecated in Vue 3.1; the example still uses it to swap the rendered component.

```vue
<template>
<tr v-is="'MyRow'"></tr>
</template>
```

## Good

The `is` attribute with the `vue:` prefix, or `<component :is>`, selects the component instead.

```vue
<template>
<tr is="vue:MyRow"></tr>
<component :is="MyRow" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_is.rs#L27) · [All rules](../all.md)
