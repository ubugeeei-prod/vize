---
title: "vue/warn-custom-directive"
description: "Warn about custom directives that need registration."
---

# `vue/warn-custom-directive`

Warn about custom directives that need registration.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

List exact globally registered directive names without the v- prefix. Local script-setup vName bindings are still discovered automatically. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.

## Bad

`v-focus` requires a project-specific directive implementation, so this optional convention reports it.

```vue annotate="remove:2"
<template>
<button type="button" v-focus>Focus</button>
</template>
```

## Good

`v-show` is a built-in Vue directive and needs no project registration.

```vue annotate="add:2"
<template>
<button type="button" v-show="visible">Copy</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/warn-custom-directive": "warn"
    },
    "ruleOptions": {
      "vue/warn-custom-directive": {
        "ignore": [
          "hover-tooltip"
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
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L49)
