---
title: "script/prefer-use-id"
description: "Recommend using useId() for generating unique IDs (Vue 3.5+)."
---

# `script/prefer-use-id`

Recommend using useId() for generating unique IDs (Vue 3.5+).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`id` contains `Math.random()`, so the generated input/label identifier can differ between server and client rendering. Its ID-named binding is the rule’s recognized generation context.

```vue annotate="remove:2"
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

## Good

Vue 3.5+ `useId()` generates the identifier, and both `:for` and `:id` continue reading the same binding instead of independently generating random values.

```vue annotate="add:2,3"
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-use-id": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L45)
