---
title: "vue/require-prop-comment"
description: "Require a JSDoc comment on each declared prop."
---

# `vue/require-prop-comment`

Require a JSDoc comment on each declared prop.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The size prop has no JSDoc comment.

```vue annotate="remove:2"
<script setup lang="ts">
const { size } = defineProps<{ size: string }>();
</script>
<template>
  <p>{{ size }}</p>
</template>
```

## Good

The size prop documents its meaning with a JSDoc comment.

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
const { size } = defineProps<{
  /** The rendered size. */
  size: string;
}>();
</script>
<template>
  <p>{{ size }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-prop-comment": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_prop_comment.rs#L18)
