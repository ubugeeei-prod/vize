---
title: "vue/no-deprecated-functional-template"
description: "Disallow the `functional` attribute on the SFC `<template>`."
---

# `vue/no-deprecated-functional-template`

Disallow the `functional` attribute on the SFC `<template>`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The SFC template has the removed functional attribute and reads the old props context.

```vue annotate="remove:1,2"
<template functional>
  <div>{{ props.msg }}</div>
</template>
```

## Good

The ordinary template omits functional and reads the component binding msg directly.

```vue annotate="add:1,2"
<template>
  <div>{{ msg }}</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-functional-template": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57)
