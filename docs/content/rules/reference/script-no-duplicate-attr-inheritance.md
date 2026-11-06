---
title: "script/no-duplicate-attr-inheritance"
description: "Flag a component that applies its fallthrough attributes twice."
---

# `script/no-duplicate-attr-inheritance`

Flag a component that applies its fallthrough attributes twice.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The component keeps automatic attribute inheritance and also forwards `$attrs` to its button. The same fallthrough attributes can be applied twice.

```vue annotate="remove:1"
<script setup></script>
<template><div><button v-bind="$attrs">Save</button></div></template>
```

## Good

`inheritAttrs: false` disables automatic inheritance, leaving the explicit button binding as the single forwarding path.

```vue annotate="add:1"
<script setup>defineOptions({ inheritAttrs: false })</script>
<template><div><button v-bind="$attrs">Save</button></div></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-duplicate-attr-inheritance": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L35)
