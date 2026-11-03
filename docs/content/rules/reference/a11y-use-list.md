---
title: "a11y/use-list"
description: "Suggest using list elements for bullet-like text."
---

# `a11y/use-list`

Suggest using list elements for bullet-like text.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The task items are separate paragraphs with typed dash markers rather than list elements.

```vue annotate="remove:2,3"
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

## Good

An unordered list and list items express the same tasks with list semantics.

```vue annotate="add:2,3,4,5"
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
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
        "a11y/use-list": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36)
