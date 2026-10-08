---
title: "a11y/aria-props"
---

# `a11y/aria-props`

Disallow invalid ARIA attributes

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
        "a11y/aria-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`aria-lable` is misspelled and is not a supported ARIA attribute.

```vue
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

## Good

The supported `aria-label` attribute supplies the button name.

```vue
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) · [All rules](../all.md)
