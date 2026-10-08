---
title: "script/prefer-import-from-vue"
---

# `script/prefer-import-from-vue`

Prefer importing from 'vue' instead of internal packages

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-import-from-vue": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`ref` and `h` are imported from the internal `@vue/runtime-core` and `@vue/runtime-dom` packages rather than the public `vue` package.

```vue
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

## Good

Both helpers are imported together from `vue`, using the public package entry point instead of either internal package.

```vue
<script setup lang="ts">
import { ref, h } from 'vue'
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L32) · [All rules](../all.md)
