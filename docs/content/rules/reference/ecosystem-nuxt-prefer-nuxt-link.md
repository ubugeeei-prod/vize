---
title: "ecosystem/nuxt-prefer-nuxt-link"
description: "Prefer NuxtLink for internal application links."
---

# `ecosystem/nuxt-prefer-nuxt-link`

Prefer NuxtLink for internal application links.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The internal settings destination uses a plain anchor in a Nuxt application.

```vue annotate="remove:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

## Good

NuxtLink handles the same internal destination through the Nuxt router.

```vue annotate="add:2"
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
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
        "ecosystem/nuxt-prefer-nuxt-link": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Ecosystem and Nuxt rules](../ecosystem.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14)
