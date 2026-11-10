---
title: "vue/use-unique-element-ids"
description: "Enforce unique element IDs using useId() instead of static literals."
---

# `vue/use-unique-element-ids`

Enforce unique element IDs using useId() instead of static literals.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The literal `email` ID is reused by every instance of this component, which can misdirect its label when several instances are rendered.

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

## Good

`useId()` produces the instance's `emailId`; bind the same value to the label's `for` and the input's `id`.

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/use-unique-element-ids": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L54)
