---
title: "vue/no-template-lang"
description: "Discourage lang attribute on template block."
---

# `vue/no-template-lang`

Discourage lang attribute on template block.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The template selects Pug through lang, so the HTML-only policy reports the authored block.

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

## Good

An ordinary HTML template omits lang and uses the paragraph directly.

```vue annotate="add:1,2"
<template>
<p>Notice</p>
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
        "vue/no-template-lang": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L11)
