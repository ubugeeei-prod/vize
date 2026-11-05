---
title: "vue/no-unsafe-url"
description: "Warn about potentially unsafe URL bindings."
---

# `vue/no-unsafe-url`

Warn about potentially unsafe URL bindings.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The anchor destination begins with the executable javascript: scheme.

```vue annotate="remove:2"
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

## Good

The anchor uses the ordinary local /next navigation destination.

```vue annotate="add:2"
<template>
<a href="/next">Continue</a>
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
        "vue/no-unsafe-url": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55)
