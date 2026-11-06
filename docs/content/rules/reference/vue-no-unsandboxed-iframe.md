---
title: "vue/no-unsandboxed-iframe"
description: "Require a sandbox attribute on iframe elements."
---

# `vue/no-unsandboxed-iframe`

Require a sandbox attribute on iframe elements.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The embedded frame has no sandbox attribute limiting its capabilities.

```vue annotate="remove:2"
<template>
<iframe src="/embed"></iframe>
</template>
```

## Good

sandbox applies restrictions; allow-scripts explicitly opts into that one capability when needed.

```vue annotate="add:2,3"
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unsandboxed-iframe": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32)
