---
title: "vapor/require-vapor-attribute"
description: "Suggest adding vapor attribute to script setup."
---

# `vapor/require-vapor-attribute`

Suggest adding vapor attribute to script setup.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Not implemented for SFC lint | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

> [!WARNING]
> Current support: `no-sfc-finding`. Vize accepts this rule ID in configuration, but the SFC linter does not report it yet. Good shows the intended convention.

This rule is a placeholder with an empty callback. Adding vapor selects Vapor compilation; the current linter does not report this catalog ID for its absence.

## Bad

The script setup block lacks the Vapor compilation attribute. This is an intended convention: the current empty rule callback does not diagnose it.

```vue annotate="remove:1"
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

## Good

Adding vapor selects Vapor compilation. It demonstrates the intended repair and does not imply that the current linter emits this catalog rule.

```vue annotate="add:1"
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

## Configured ID (currently no SFC finding)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vapor rules](../vapor.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17)
