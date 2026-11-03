---
title: "ecosystem/vue-router-prefer-named-link"
description: "Prefer named route objects over static path strings in RouterLink."
---

# `ecosystem/vue-router-prefer-named-link`

Prefer named route objects over static path strings in RouterLink.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The RouterLink destination is a literal path rather than a named route.

```vue annotate="remove:2"
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

## Good

The bound route object identifies the destination by its settings route name.

```vue annotate="add:2"
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-router-prefer-named-link": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Ecosystem and Nuxt rules](../ecosystem.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15)
