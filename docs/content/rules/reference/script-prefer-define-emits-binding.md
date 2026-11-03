---
title: "script/prefer-define-emits-binding"
description: "Prefer the captured defineEmits function over template $emit."
---

# `script/prefer-define-emits-binding`

Prefer the captured defineEmits function over template $emit.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | Yes | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The template calls the component-only `$emit` helper even though `<script setup>` already captures the typed function returned by `defineEmits`.

```vue annotate="remove:4"
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="$emit('save', 1)">Save</button></template>
```

## Good

The template calls the captured `emit` binding, so event names and payloads go through the component's `defineEmits` contract.

```vue annotate="add:4"
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="emit('save', 1)">Save</button></template>
```

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

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/prefer_define_emits_binding.rs#L16)
