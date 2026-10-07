---
title: "ssr/no-browser-globals-in-ssr"
---

# `ssr/no-browser-globals-in-ssr`

Disallow browser-only globals in SSR context

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
        "ssr/no-browser-globals-in-ssr": "warn"
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
const width = window.innerWidth;
</script>
```

## Good

```vue
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L158) · [All rules](../all.md)
