---
title: "vue/no-restricted-html-elements"
description: "disallow specific elements."
---

# `vue/no-restricted-html-elements`

disallow specific elements.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The template uses the forbidden marquee element.

```vue annotate="remove:2"
<template>
  <marquee>x</marquee>
</template>
```

## Good

The template uses p.

```vue annotate="add:2"
<template>
  <p>x</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-html-elements": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-html-elements": [
        "marquee"
      ]
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_html.rs#L13)
