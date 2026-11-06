---
title: "script/no-next-tick"
description: "Disallow nextTick() usage in Vapor-oriented components."
---

# `script/no-next-tick`

Disallow nextTick() usage in Vapor-oriented components.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | None | No | None |

Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts.

## Bad

The Vapor-oriented component imports and awaits `nextTick`, introducing the DOM-flush scheduling dependency that this migration rule rejects.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

## Good

The input is obtained through `useTemplateRef` and focused at `onMounted`. The explicit mount boundary replaces the example’s `nextTick` dependency.

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-next-tick": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40)
