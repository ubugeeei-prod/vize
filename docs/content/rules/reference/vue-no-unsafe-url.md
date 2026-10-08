---
title: "vue/no-unsafe-url"
---

# `vue/no-unsafe-url`

Warn about potentially unsafe URL bindings

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
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
        "vue/no-unsafe-url": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The anchor destination begins with the executable javascript: scheme.

```vue
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

## Good

The anchor uses the ordinary local /next navigation destination.

```vue
<template>
<a href="/next">Continue</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [All rules](../all.md)
