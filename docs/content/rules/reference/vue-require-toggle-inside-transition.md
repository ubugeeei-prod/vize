---
title: "vue/require-toggle-inside-transition"
description: "Require a toggle on the element wrapped by `<transition>`."
---

# `vue/require-toggle-inside-transition`

Require a toggle on the element wrapped by `<transition>`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The static child inside `<Transition>` has no conditional visibility or dynamic selection to trigger an enter/leave change.

```vue annotate="remove:3"
<template>
<transition>
  <div>content</div>
</transition>
</template>
```

## Good

`v-if="show"` changes whether the child exists, giving the transition an enter/leave boundary.

```vue annotate="add:3"
<template>
<transition>
  <div v-if="show">content</div>
</transition>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-toggle-inside-transition": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L49)
