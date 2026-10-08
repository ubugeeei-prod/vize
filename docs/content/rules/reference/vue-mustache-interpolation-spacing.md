---
title: "vue/mustache-interpolation-spacing"
---

# `vue/mustache-interpolation-spacing`

Enforce consistent spacing inside mustache interpolations

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: Available for supported findings  
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
        "vue/mustache-interpolation-spacing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The text interpolation is missing a space at one or both delimiter boundaries.

```vue
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

## Good

Spaces separate the expression from both opening and closing mustache delimiters.

```vue
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) · [All rules](../all.md)
