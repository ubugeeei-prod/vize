---
title: "vue/this-in-template"
---

# `vue/this-in-template`

Disallow `this.` in template expressions

[Bad](#bad) · [Good](#good)

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
        "vue/this-in-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

Template expressions explicitly access `this.message`, `this.className`, and `this.handleClick`, although Vue exposes those bindings directly.

```vue
<template>
<div>{{ this.message }}</div>
<div :class="this.className"></div>
<button @click="this.handleClick()"></button>
</template>
```

## Good

Use `message`, `className`, and `handleClick` directly. The literal string `'this.is.a.string'` stays unchanged because it is not a member access.

```vue
<template>
<div>{{ message }}</div>
<div :class="className"></div>
<button @click="handleClick()"></button>
<div>{{ 'this.is.a.string' }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L33) · [All rules](../all.md)
