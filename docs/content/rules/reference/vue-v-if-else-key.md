---
title: "vue/v-if-else-key"
description: "Require keys on repeated conditional components."
---

# `vue/v-if-else-key`

Require keys on repeated conditional components.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Repeated conditional `Panel` instances have no keys.

```vue annotate="remove:2"
<template>
  <section><Panel v-if="ready" /><Panel v-else /></section>
</template>
```

## Good

Each conditional component has a stable unique key.

```vue annotate="add:2"
<template>
  <section><Panel v-if="ready" key="ready" /><Panel v-else key="fallback" /></section>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-if-else-key": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules/v_if_else_key.rs#L17)
