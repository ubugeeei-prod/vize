---
title: "script/no-expose-after-await"
---

# `script/no-expose-after-await`

Disallow expose() and defineExpose() after an await

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-expose-after-await": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`defineExpose()` runs after a top-level `await`, when there is no current component instance, so nothing is exposed.

```vue
<script setup lang="ts">
await loadData()
defineExpose({ reload })
</script>
```

## Good

`defineExpose()` runs before the first `await`, while the component instance is still current.

```vue
<script setup lang="ts">
defineExpose({ reload })
await loadData()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_expose_after_await.rs#L34) · [All rules](../all.md)
