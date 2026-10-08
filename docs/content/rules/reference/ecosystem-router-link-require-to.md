---
title: "ecosystem/router-link-require-to"
---

# `ecosystem/router-link-require-to`

Require a `to` target on RouterLink and NuxtLink components

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

A single SFC root link may inherit its target from parent attributes. This example uses a nested link, whose target must be explicit.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/router-link-require-to": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The nested RouterLink has no `to` destination; it cannot rely on root attribute fallthrough.

```vue
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

## Good

`to="/settings"` explicitly supplies the nested link destination.

```vue
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) · [All rules](../all.md)
