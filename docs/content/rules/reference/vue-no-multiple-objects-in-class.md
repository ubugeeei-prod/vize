---
title: "vue/no-multiple-objects-in-class"
---

# `vue/no-multiple-objects-in-class`

Disallow multiple object literals inside a :class array binding

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
        "vue/no-multiple-objects-in-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

A class array contains two top-level object literals that can be merged.

```vue
<template>
<div :class="[{ a }, { b }]"></div>
<div :class="[{ active: isActive }, { error: hasError }]"></div>
</template>
```

## Good

One object contains the class conditions; arrays with one object and a string or with non-literal entries remain allowed.

```vue
<template>
<div :class="{ a, b }"></div>
<div :class="[{ active: isActive }, 'static']"></div>
<div :class="[foo, bar]"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) · [All rules](../all.md)
