---
title: "vue/warn-custom-block"
description: "Warn about custom blocks in SFC files."
---

# `vue/warn-custom-block`

Warn about custom blocks in SFC files.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The SFC contains an `<i18n>` custom block, which needs an external integration beyond ordinary template/script/style processing.

```vue annotate="remove:1,2,3,4"
<i18n>
{ "en": { "hello": "Hello" } }
</i18n>

<template>
  <p>{{ hello }}</p>
</template>
```

## Good

The example uses standard template and script-setup blocks. This optional portability warning does not mean every custom block is invalid Vue.

```vue annotate="add:4,5,6,7"
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/warn-custom-block": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50)
