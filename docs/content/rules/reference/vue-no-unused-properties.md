---
title: "vue/no-unused-properties"
description: "Disallow unused properties defined in defineProps."
---

# `vue/no-unused-properties`

Disallow unused properties defined in defineProps.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The component declares description as a prop but renders only title.

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
```

## Good

Both declared props are referenced by the template.

```vue annotate="add:7"
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ description }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-properties": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L95)
