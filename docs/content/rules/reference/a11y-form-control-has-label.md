---
title: "a11y/form-control-has-label"
---

# `a11y/form-control-has-label`

Require form controls to have associated labels

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
        "a11y/form-control-has-label": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The search input has no label identifying what the user should enter.

```vue
<template>
  <input type="search" />
</template>
```

## Good

Wrapping the input in a label associates the visible `Search` text with the control.

```vue
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) · [All rules](../all.md)
