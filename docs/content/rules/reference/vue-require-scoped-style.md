---
title: "vue/require-scoped-style"
description: "Require scoped attribute on style tags."
---

# `vue/require-scoped-style`

Require scoped attribute on style tags.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The `.button` style is unscoped and can affect matching elements outside this component.

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

## Good

Adding `scoped` applies Vue's component scope to the same selector and declarations.

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
}
</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/require-scoped-style": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49)
