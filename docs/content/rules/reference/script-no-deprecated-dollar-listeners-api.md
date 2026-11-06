---
title: "script/no-deprecated-dollar-listeners-api"
description: "Disallow the $listeners instance property removed in Vue 3 (merged into $attrs)."
---

# `script/no-deprecated-dollar-listeners-api`

Disallow the $listeners instance property removed in Vue 3 (merged into $attrs).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The script reads the removed `$listeners` property. The template also forwards the removed component-context `$listeners` value; proven setup values and template-local bindings with that name are separate values.

```vue annotate="remove:4,5,9"
<script>
export default {
  mounted() {
    const vm = this
    return vm.$listeners
  }
}
</script>
<template><Child v-on="$listeners" /></template>
```

## Good

The script uses `$attrs`, and `v-bind="$attrs"` forwards Vue 3 attributes and listeners. Script warnings retain their existing broad receiver/reference coverage.

```vue annotate="add:4,8"
<script>
export default {
  mounted() {
    return this.$attrs
  }
}
</script>
<template><Child v-bind="$attrs" /></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-dollar-listeners-api": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L54)
