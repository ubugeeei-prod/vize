---
title: "vue/valid-v-memo"
description: "Enforce valid `v-memo` directives."
---

# `vue/valid-v-memo`

Enforce valid `v-memo` directives.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Bare `v-memo` gives Vue no dependency expression for deciding when to reuse the subtree.

```vue annotate="remove:2"
<template>
  <div v-memo></div>
</template>
```

## Good

`v-memo="[valueA, valueB]"` supplies the dependency array used for memoization.

```vue annotate="add:2"
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-memo": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L32)
