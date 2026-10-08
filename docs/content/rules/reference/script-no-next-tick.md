---
title: "script/no-next-tick"
---

# `script/no-next-tick`

Disallow nextTick() usage in Vapor-oriented components

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The Vapor-oriented component imports and awaits `nextTick`, introducing the DOM-flush scheduling dependency that this migration rule rejects.

```vue
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

## Good

The input is obtained through `useTemplateRef` and focused at `onMounted`. The explicit mount boundary replaces the example’s `nextTick` dependency.

```vue
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [All rules](../all.md)
