---
title: "script/define-props-destructuring"
description: "Enforce consistent style for defineProps destructuring in <script setup>."
---

# `script/define-props-destructuring`

Enforce consistent style for defineProps destructuring in &lt;script setup&gt;.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | [Rule options](../options.md) |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The bare typed `defineProps` call declares `foo` without binding it through the default destructuring policy.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ foo: string }>()
</script>
```

## Good

The object pattern binds `foo` and `bar` directly and gives the optional `bar` a default. This relies on Vue 3.5+ reactive props destructuring; the configurable `never` mode prefers the opposite form.

```vue annotate="add:2"
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/define-props-destructuring": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L31)
