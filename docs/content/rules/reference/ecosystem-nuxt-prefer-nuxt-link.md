---
title: "ecosystem/nuxt-prefer-nuxt-link"
---

# `ecosystem/nuxt-prefer-nuxt-link`

Prefer NuxtLink for internal application links

Default severity: `warning`  
Presets: `nuxt`  
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
        "ecosystem/nuxt-prefer-nuxt-link": "warn"
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
  <a href="/settings">Settings</a>
</template>
```

## Good

```vue
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) · [All rules](../all.md)
