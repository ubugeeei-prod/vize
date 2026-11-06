---
title: "vue/no-unused-setup-bindings"
description: "Disallow unread script setup bindings."
---

# `vue/no-unused-setup-bindings`

Disallow unread script setup bindings.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The script setup message binding is never read by the template.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

## Good

The paragraph interpolates message, using the declared binding.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-setup-bindings": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19)
