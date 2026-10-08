---
title: "ssr/no-hydration-mismatch"
---

# `ssr/no-hydration-mismatch`

Disallow non-deterministic values that cause hydration mismatch

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The template evaluates `Math.random()` during rendering, so the server and client can produce different text for the same paragraph.

```vue
<template>
  <p>{{ Math.random() }}</p>
</template>
```

## Good

The paragraph renders the stable `seed` state instead of a fresh random result. In this Nuxt-style example, `useState` supplies the shared state and the initializer is the constant `"stable"`.

```vue
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) · [All rules](../all.md)
