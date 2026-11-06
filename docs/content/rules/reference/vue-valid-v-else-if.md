---
title: "vue/valid-v-else-if"
description: "Enforce valid `v-else-if` directives."
---

# `vue/valid-v-else-if`

Enforce valid `v-else-if` directives.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The examples leave `v-else-if` without a preceding conditional branch or a condition, or give it an argument or modifier.

```vue annotate="remove:2,3,4,5"
<template>
<div v-else-if="foo"></div> <!-- without preceding v-if -->
<div v-if="foo"></div><div v-else-if></div>
<div v-if="foo"></div><div v-else-if:arg="bar"></div>
<div v-if="foo"></div><div v-else-if.mod="bar"></div>
</template>
```

## Good

Place `v-else-if` with a condition immediately after the `v-if` or `v-else-if` branch it continues.

```vue annotate="add:2,3"
<template>
<div v-if="foo"></div>
<div v-else-if="bar"></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-else-if": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else_if.rs#L34)
