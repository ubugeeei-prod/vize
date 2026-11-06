---
title: "ssr/no-hydration-mismatch"
description: "Disallow non-deterministic values that cause hydration mismatch."
---

# `ssr/no-hydration-mismatch`

Disallow non-deterministic values that cause hydration mismatch.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The template evaluates `Math.random()` during rendering, so the server and client can produce different text for the same paragraph.

```vue annotate="remove:2"
<template>
  <p>{{ Math.random() }}</p>
</template>
```

## Good

The paragraph renders the stable `seed` state instead of a fresh random result. In this Nuxt-style example, `useState` supplies the shared state and the initializer is the constant `"stable"`.

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ssr/no-hydration-mismatch": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [SSR rules](../ssr.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122)
