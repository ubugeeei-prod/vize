---
title: "vue/comment-directive"
description: "Support comment directives in `<template>`."
---

# `vue/comment-directive`

Support comment directives in `<template>`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

Unused template suppression comments are reported only when `reportUnusedDisableDirectives: true` is enabled. The example removes an unused comment; comments that actually suppress a finding remain useful. Listed rule names count only when Vize executes those rules.

## Bad

The blanket disable comment suppresses no finding on the following `div`, so unused-directive reporting rejects it.

```vue annotate="remove:2"
<template>
  <!-- eslint-disable-next-line -->
  <div />
</template>
```

## Good

Remove the unused comment and leave the element unchanged, keeping lint checks active on that line.

```vue
<template>
  <div />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/comment-directive": "error"
    },
    "reportUnusedDisableDirectives": true
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/comment_directive.rs#L33)
