---
title: "vue/require-expose"
description: "Require declare public properties using `expose`."
---

# `vue/require-expose`

Require declare public properties using `expose`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

This check concerns Options API component objects. An explicit empty `expose` list is valid. Setup's `expose` call and returning a render function also satisfy the check; mixins are exempt. The script pass does not inspect a sibling `<script setup>` block, and `defineExpose` is not this rule's target.

## Bad

The Options API component has a `focus` method without explicitly declaring its public instance interface.

```vue annotate="remove:2"
<script lang="ts">
export default { methods: { focus() {} } };
</script>
```

## Good

An `expose` list explicitly makes `focus` available through the public component instance.

```vue annotate="add:2"
<script lang="ts">
export default { expose: ["focus"], methods: { focus() {} } };
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-expose": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_expose.rs#L24)
