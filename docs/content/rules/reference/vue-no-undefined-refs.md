---
title: "vue/no-undefined-refs"
description: "Disallow undefined variable references in templates."
---

# `vue/no-undefined-refs`

Disallow undefined variable references in templates.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

`globals` lists names a build step defines for every template, such as `unplugin-auto-import` imports. In a Nuxt project, `vize lint` reads the auto-imports from the generated `.nuxt/types/imports.d.ts` and adds them, so they need no entry; until `nuxi prepare` has generated that file, the rule is skipped with a notice. Names are exact. Options do not enable the rule. Later layers replace the list; an empty list clears inherited names.

## Bad

The template reads missing, although the script declares only message.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

## Good

The interpolation reads the existing message binding.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-undefined-refs": "warn"
    },
    "ruleOptions": {
      "vue/no-undefined-refs": {
        "globals": [
          "formatCount"
        ]
      }
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L13)
