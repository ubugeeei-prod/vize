---
title: "html/no-duplicate-class"
---

# `html/no-duplicate-class`

Disallow duplicate class names in a static class attribute

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
        "html/no-duplicate-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The static class list repeats the `btn` token.

```vue
<template>
<div class="btn btn primary">click</div>
</template>
```

## Good

The class list keeps one `btn` token and the distinct `primary` token.

```vue
<template>
<div class="btn primary">click</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) · [All rules](../all.md)
