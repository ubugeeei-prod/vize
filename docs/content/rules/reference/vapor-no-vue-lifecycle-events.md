---
title: "vapor/no-vue-lifecycle-events"
---

# `vapor/no-vue-lifecycle-events`

Disallow @vue:xxx per-element lifecycle events (not supported in Vapor)

[Bad](#bad) · [Good](#good)

Default severity: `error`  
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
        "vapor/no-vue-lifecycle-events": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The input uses the @vue:mounted template lifecycle event.

```vue
<template>
  <input @vue:mounted="focusInput" />
</template>
```

## Good

onMounted accesses the named template reference and focuses the input through the supported script lifecycle hook.

```vue
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) · [All rules](../all.md)
