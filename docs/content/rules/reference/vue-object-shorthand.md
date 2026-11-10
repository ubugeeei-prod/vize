---
title: "vue/object-shorthand"
description: "Require object shorthand in template expressions."
---

# `vue/object-shorthand`

Require object shorthand in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The object repeats `value` as both key and value.

```vue annotate="remove:2"
<template>
  <Widget :options="{ value: value }" />
</template>
```

## Good

Property shorthand keeps the same binding without repetition.

```vue annotate="add:2"
<template>
  <Widget :options="{ value }" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/object-shorthand": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/object_shorthand.rs#L10)
