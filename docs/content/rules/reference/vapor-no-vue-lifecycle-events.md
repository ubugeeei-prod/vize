---
title: "vapor/no-vue-lifecycle-events"
description: "Disallow @vue:xxx per-element lifecycle events (not supported in Vapor)."
---

# `vapor/no-vue-lifecycle-events`

Disallow @vue:xxx per-element lifecycle events (not supported in Vapor).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The input uses the @vue:mounted template lifecycle event.

```vue annotate="remove:2"
<template>
  <input @vue:mounted="focusInput" />
</template>
```

## Good

onMounted accesses the named template reference and focuses the input through the supported script lifecycle hook.

```vue annotate="add:1,2,3,4,5,6,7,8,10"
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

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vapor/no-vue-lifecycle-events": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vapor rules](../vapor.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34)
