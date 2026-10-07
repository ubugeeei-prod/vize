---
title: "html/no-consecutive-br"
---

# `html/no-consecutive-br`

Disallow consecutive <br> elements

Default severity: `warning`  
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
        "html/no-consecutive-br": "warn"
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
  <p>First line<br /><br />Second block</p>
</template>
```

## Good

```vue
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) · [All rules](../all.md)
