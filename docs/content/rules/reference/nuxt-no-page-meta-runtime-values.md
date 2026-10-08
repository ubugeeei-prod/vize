---
title: "nuxt/no-page-meta-runtime-values"
---

# `nuxt/no-page-meta-runtime-values`

Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `nuxt`  
Automatic fix: None; review the suggested change  
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
        "nuxt/no-page-meta-runtime-values": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`useRoute()` is evaluated immediately while building the `definePageMeta` object, although the macro hoists that metadata outside the setup runtime context.

```vue
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

## Good

`validate` receives a callback, so its `useRoute().params.id` access is deferred until the callback runs. The rule distinguishes deferred function bodies from eager metadata values.

```vue
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) · [All rules](../all.md)
