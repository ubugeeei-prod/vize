---
title: "vue/no-useless-concat"
description: "Disallow useless string concatenation in template expressions."
---

# `vue/no-useless-concat`

Disallow useless string concatenation in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Two string literals are concatenated at runtime.

```vue annotate="remove:2"
<template>
  <p>{{ "hello" + " world" }}</p>
</template>
```

## Good

One literal represents the same text.

```vue annotate="add:2"
<template>
  <p>{{ "hello world" }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-useless-concat": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L153)
