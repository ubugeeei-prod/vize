---
title: "vue/no-restricted-component-names"
description: "disallow specific component names."
---

# `vue/no-restricted-component-names`

disallow specific component names.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The component name is the forbidden RestrictedShowcase.

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "RestrictedShowcase" });
</script>
<template>
  <p>ok</p>
</template>
```

## Good

The component name is Widget.

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "Widget" });
</script>
<template>
  <p>ok</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-component-names": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-component-names": [
        "RestrictedShowcase"
      ]
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/component_names.rs#L12)
