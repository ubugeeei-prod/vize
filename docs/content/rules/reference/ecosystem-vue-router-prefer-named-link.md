---
title: "ecosystem/vue-router-prefer-named-link"
---

# `ecosystem/vue-router-prefer-named-link`

Prefer named route objects over static path strings in RouterLink

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `ecosystem`  
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
        "ecosystem/vue-router-prefer-named-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The RouterLink destination is a literal path rather than a named route.

```vue
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

## Good

The bound route object identifies the destination by its settings route name.

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) · [All rules](../all.md)
