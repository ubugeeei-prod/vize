---
title: "vue/no-template-target-blank"
description: "Disallow target=\"_blank\" without rel=\"noopener noreferrer\"."
---

# `vue/no-template-target-blank`

Disallow target="_blank" without rel="noopener noreferrer".

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The external link opens a new browsing context without the expected rel protection.

```vue annotate="remove:2"
<template>
<a href="https://example.com" target="_blank">x</a>
</template>
```

## Good

The same link includes noopener noreferrer alongside target=_blank.

```vue annotate="add:2"
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
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
        "vue/no-template-target-blank": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33)
