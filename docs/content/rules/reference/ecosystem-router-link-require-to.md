---
title: "ecosystem/router-link-require-to"
description: "Require a `to` target on RouterLink and NuxtLink components."
---

# `ecosystem/router-link-require-to`

Require a `to` target on RouterLink and NuxtLink components.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `ecosystem` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

A single SFC root link may inherit its target from parent attributes. This example uses a nested link, whose target must be explicit.

## Bad

The nested RouterLink has no `to` destination; it cannot rely on root attribute fallthrough.

```vue annotate="remove:2"
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

## Good

`to="/settings"` explicitly supplies the nested link destination.

```vue annotate="add:2"
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/router-link-require-to": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Ecosystem and Nuxt rules](../ecosystem.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14)
