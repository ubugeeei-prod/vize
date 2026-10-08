---
title: "a11y/interactive-supports-focus"
---

# `a11y/interactive-supports-focus`

Require interactive role elements to be focusable

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
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

Giving a `span` the button role and a click handler does not make the element keyboard-focusable.

```vue
<template>
  <span role="button" @click="open">Open</span>
</template>
```

## Good

The native button is focusable and retains the same `open` action.

```vue
<template>
  <button type="button" @click="open">Open</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) · [All rules](../all.md)
