---
title: "a11y/alt-text"
---

# `a11y/alt-text`

Require alternative text for media elements

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
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The image submit control supplies only its image URL; it has no `alt` text describing the action.

```vue
<template>
  <input type="image" src="/submit.png" />
</template>
```

## Good

`alt="Submit search"` gives the image control an accessible name that describes submitting the search.

```vue
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) · [All rules](../all.md)
