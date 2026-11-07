---
title: "vue/no-negated-v-if-condition"
description: "Disallow a negated v-if condition when the chain has a v-else."
---

# `vue/no-negated-v-if-condition`

Disallow a negated v-if condition when the chain has a v-else.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The paired v-if and v-else branches begin with a negated condition.

```vue
<template>
<div v-if="!ok">A</div>
<div v-else>B</div>
</template>
```

## Good

A positive ok condition comes first; when inverting a condition, place the original opposite branch first. A lone negated v-if and !== comparisons remain allowed.

```vue annotate="add:2,3,4,6,7"
<template>
<div v-if="ok">B</div>
<div v-else>A</div>

<div v-if="!ok">A</div>

<div v-if="a !== b">A</div>
<div v-else>B</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-negated-v-if-condition": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L41)
