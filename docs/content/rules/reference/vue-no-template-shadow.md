---
title: "vue/no-template-shadow"
description: "Disallow variable names that shadow variables in outer scope."
---

# `vue/no-template-shadow`

Disallow variable names that shadow variables in outer scope.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

The check compares `v-for`, `v-slot` (including `#name`), and legacy scoped-slot bindings within the template. It reports the identifier that hides an earlier binding, including declarations on the same element in attribute order. Sibling elements have independent scopes. Property keys and default-value references are not declarations, and `v-pre` makes its subtree inert.

Script and component property names are outside the current check: a single template binding does not produce a warning merely because a script binding has the same name. Diagnostic ranges cover the authored identifier, including escaped spellings and HTML entities.

## Bad

The inner v-for declares item again and hides the outer item binding inside the nested loop.

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

## Good

The inner loop declares child, leaving item available for the outer row and child for the nested row.

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-template-shadow": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L35)
