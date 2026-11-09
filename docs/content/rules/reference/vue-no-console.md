---
title: "vue/no-console"
description: "Disallow console access in template expressions."
---

# `vue/no-console`

Disallow console access in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The template event expression calls `console.log`.

```vue annotate="remove:2"
<template>
  <button @click="console.log(value)">Save</button>
</template>
```

## Good

The template delegates logging to a script binding.

```vue annotate="add:2"
<template>
  <button @click="logValue">Save</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-console": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L97)
