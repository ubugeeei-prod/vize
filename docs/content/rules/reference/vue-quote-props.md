---
title: "vue/quote-props"
description: "Disallow unnecessary quoted object properties in template expressions."
---

# `vue/quote-props`

Disallow unnecessary quoted object properties in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The identifier-safe `name` key is unnecessarily quoted.

```vue annotate="remove:2"
<template>
  <Widget :options='{ "name": value }' />
</template>
```

## Good

The property uses its unquoted identifier form.

```vue annotate="add:2"
<template>
  <Widget :options="{ name: value }" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/quote-props": "warn"
    },
    "ruleOptions": {
      "vue/quote-props": "as-needed"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/quote_props.rs#L10)
