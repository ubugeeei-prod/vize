---
title: "vue/no-negated-condition"
description: "Disallow negated ternary conditions in template expressions."
---

# `vue/no-negated-condition`

Disallow negated ternary conditions in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The ternary starts with a negated condition.

```vue annotate="remove:2"
<template>
  <p>{{ !ready ? fallback : primary }}</p>
</template>
```

## Good

The positive condition and branch order communicate the choice directly.

```vue annotate="add:2"
<template>
  <p>{{ ready ? primary : fallback }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-negated-condition": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L117)
