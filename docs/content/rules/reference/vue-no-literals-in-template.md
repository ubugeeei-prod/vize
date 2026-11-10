---
title: "vue/no-literals-in-template"
description: "Disallow object, array and function literals in v-bind."
---

# `vue/no-literals-in-template`

Disallow object, array and function literals in v-bind.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The object literal creates a new binding value during rendering.

```vue annotate="remove:2"
<template>
  <Widget :options="{ eager: true }" />
</template>
```

## Good

A stable script binding supplies the options object.

```vue annotate="add:2"
<template>
  <Widget :options="options" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-literals-in-template": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules/no_literals_in_template.rs#L10)
