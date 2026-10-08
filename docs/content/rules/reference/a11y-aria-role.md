---
title: "a11y/aria-role"
---

# `a11y/aria-role`

Elements with ARIA roles must use a valid, non-abstract ARIA role

[Bad](#bad) · [Good](#good)

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
        "a11y/aria-role": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`datepicker` is not a recognized ARIA role for this section.

```vue
<template>
  <section role="datepicker">...</section>
</template>
```

## Good

The section uses the recognized `dialog` role and a label describing the date selection.

```vue
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) · [All rules](../all.md)
