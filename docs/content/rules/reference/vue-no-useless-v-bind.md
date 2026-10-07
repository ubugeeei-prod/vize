---
title: "vue/no-useless-v-bind"
---

# `vue/no-useless-v-bind`

Disallow a v-bind whose value is a plain string literal

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
        "vue/no-useless-v-bind": "warn"
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
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

## Good

```vue
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) · [All rules](../all.md)
