---
title: "vue/no-restricted-v-bind"
description: "Disallow restricted v-bind arguments."
---

# `vue/no-restricted-v-bind`

Disallow restricted v-bind arguments.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

`:v-focus` binds a directive-shaped argument as an attribute.

```vue annotate="remove:2"
<template>
  <input :v-focus="enabled" />
</template>
```

## Good

`v-focus` uses the directive syntax directly.

```vue annotate="add:2"
<template>
  <input v-focus="enabled" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-v-bind": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules/no_restricted_v_bind.rs#L9)
