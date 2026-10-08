---
title: "a11y/no-access-key"
---

# `a11y/no-access-key`

Disallow the use of the accesskey attribute

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
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The `accesskey="s"` shortcut may conflict with browser or assistive-technology shortcuts.

```vue
<template>
  <button accesskey="s">Save</button>
</template>
```

## Good

Removing `accesskey` keeps the ordinary Save button available.

```vue
<template>
  <button>Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) · [All rules](../all.md)
