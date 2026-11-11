---
title: "vue/block-lang"
description: "Disallow languages other than the configured lang on an SFC block."
---

# `vue/block-lang`

Disallow languages other than the configured lang on an SFC block.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The script block omits the configured lang attribute.

```vue annotate="remove:1"
<script setup>
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## Good

The script block sets lang to the configured language.

```vue annotate="add:1"
<script setup lang="ts">
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/block-lang": "warn"
    },
    "ruleOptions": {
      "vue/block-lang": {
        "script": {
          "lang": "ts",
          "allowNoLang": false
        }
      }
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/block_lang.rs#L13)
