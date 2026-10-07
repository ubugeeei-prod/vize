---
title: "a11y/aria-unsupported-elements"
---

# `a11y/aria-unsupported-elements`

Disallow ARIA attributes on elements that do not support them

Default severity: `error`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "a11y/aria-unsupported-elements": "error"
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
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

## Good

```vue
<template>
  <meta charset="utf-8" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) · [All rules](../all.md)
