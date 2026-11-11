---
title: "vue/no-restricted-v-on"
description: "disallow specific argument in `v-on`."
---

# `vue/no-restricted-v-on`

disallow specific argument in `v-on`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The button listens for the forbidden legacy-click.

```vue annotate="remove:2"
<template>
  <button type="button" @legacy-click="save">x</button>
</template>
```

## Good

The button listens for click.

```vue annotate="add:2"
<template>
  <button type="button" @click="save">x</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-v-on": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-v-on": [
        "legacy-click"
      ]
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_v_on.rs#L13)
