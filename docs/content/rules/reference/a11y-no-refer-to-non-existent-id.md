---
title: "a11y/no-refer-to-non-existent-id"
---

# `a11y/no-refer-to-non-existent-id`

Disallow references to non-existent IDs

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
        "a11y/no-refer-to-non-existent-id": "warn"
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
  <button aria-labelledby="save-label">Save</button>
</template>
```

## Good

```vue
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) · [All rules](../all.md)
