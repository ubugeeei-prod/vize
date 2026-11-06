---
title: "script/no-deprecated-dollar-scopedslots-api"
description: "Disallow the $scopedSlots instance property removed in Vue 3 (use $slots)."
---

# `script/no-deprecated-dollar-scopedslots-api`

Disallow the $scopedSlots instance property removed in Vue 3 (use $slots).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`this.$scopedSlots`, `ctx.$scopedSlots`, and the bare `$scopedSlots` reference use the Vue 2 scoped-slot API removed in Vue 3.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

## Good

Replacing `$scopedSlots` with `$slots` uses the unified slot surface. The example removes the deprecated spelling rather than establishing a setup context for the receivers.

```vue annotate="add:2,3,4"
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-dollar-scopedslots-api": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L45)
