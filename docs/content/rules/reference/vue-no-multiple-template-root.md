---
title: "vue/no-multiple-template-root"
description: "Disallow multiple root nodes in a template."
---

# `vue/no-multiple-template-root`

Disallow multiple root nodes in a template.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

Enable only for a single-root contract. Vue 3 normally supports fragments.

## Bad

The opt-in single-root convention sees two sibling paragraphs at the template root.

```vue annotate="remove:2,3"
<template>
<p>First</p>
<p>Second</p>
</template>
```

## Good

A section wraps the paragraphs into one root; enable this convention only when a single-root contract is intended.

```vue annotate="add:2"
<template>
<section><p>First</p><p>Second</p></section>
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
        "vue/no-multiple-template-root": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L23)
