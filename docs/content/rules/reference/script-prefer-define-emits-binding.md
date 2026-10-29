---
title: "script/prefer-define-emits-binding"
---

# `script/prefer-define-emits-binding`

Prefer the captured defineEmits function over template $emit

[Bad](#bad) · [Good](#good)

Default severity: `warning`

Presets: _none_

Automatic fix: Available for supported findings

Applies to: Vue SFCs with `<script setup>` and a compiler-provided `defineEmits` call; checks template `$emit` calls

Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/prefer-define-emits-binding": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The template calls the component-only `$emit` helper even though `<script setup>` already captures the typed function returned by `defineEmits`.

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="$emit('save', 1)">Save</button></template>
```

## Good

The template calls the captured `emit` binding, so event names and payloads go through the component's `defineEmits` contract.

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="emit('save', 1)">Save</button></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/prefer_define_emits_binding.rs#L16) · [All rules](../all.md)
