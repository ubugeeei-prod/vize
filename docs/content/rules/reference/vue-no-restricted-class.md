---
title: "vue/no-restricted-class"
description: "disallow specific classes in Vue components."
---

# `vue/no-restricted-class`

disallow specific classes in Vue components.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The element uses the forbidden legacy class.

```vue annotate="remove:2"
<template>
  <div class="legacy">x</div>
</template>
```

## Good

The element uses an allowed class.

```vue annotate="add:2"
<template>
  <div class="current">x</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-class": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-class": [
        "legacy"
      ]
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_class.rs#L13)
