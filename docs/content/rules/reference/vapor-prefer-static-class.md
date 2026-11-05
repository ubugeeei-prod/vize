---
title: "vapor/prefer-static-class"
description: "Prefer static class over dynamic class binding for string literals."
---

# `vapor/prefer-static-class`

Prefer static class over dynamic class binding for string literals.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Yes | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The class binding evaluates a constant string even though the class does not change.

```vue annotate="remove:2"
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

## Good

A static class attribute expresses the same panel classes without a binding.

```vue annotate="add:2"
<template>
  <section class="panel panel-primary">Profile</section>
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
        "vapor/prefer-static-class": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vapor rules](../vapor.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31)
