---
title: "vue/no-restricted-block"
description: "disallow specific block."
---

# `vue/no-restricted-block`

disallow specific block.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The SFC contains the forbidden i18n block.

```vue annotate="remove:4"
<template>
  <p>ok</p>
</template>
<i18n lang="json">{}</i18n>
```

## Good

The SFC keeps only the template block.

```vue
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
      "vue/no-restricted-block": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-block": [
        "i18n"
      ]
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/blocks.rs#L14)
