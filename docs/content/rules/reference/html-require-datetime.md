---
title: "html/require-datetime"
---

# `html/require-datetime`

Require datetime attribute on <time> element

[Bad](#bad) · [Good](#good)

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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The time element contains a human-readable date but no machine-readable datetime value.

```vue
<template>
  <time>May 13, 2026</time>
</template>
```

## Good

`datetime="2026-05-13"` supplies the corresponding machine-readable date.

```vue
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) · [All rules](../all.md)
