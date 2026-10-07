---
title: "nuxt/prefer-import-meta"
---

# `nuxt/prefer-import-meta`

Prefer using `import.meta.*` over `process.*`

Default severity: `error`  
Presets: `nuxt`  
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
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

```vue
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

## Good

```vue
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) · [All rules](../all.md)
