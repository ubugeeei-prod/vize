---
title: "a11y/no-aria-hidden-on-focusable"
---

# `a11y/no-aria-hidden-on-focusable`

Disallow aria-hidden="true" on focusable elements

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
        "a11y/no-aria-hidden-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The focusable Close button is hidden from the accessibility tree with `aria-hidden="true"`.

```vue
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

## Good

The button remains exposed and receives a `Close` label instead of being hidden.

```vue
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) · [All rules](../all.md)
