---
title: "vue/no-reserved-component-names"
description: "Disallow the use of reserved names as component names."
---

# `vue/no-reserved-component-names`

Disallow the use of reserved names as component names.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The component name button conflicts with a native HTML element name.

```vue annotate="remove:1,2,3,4"
<script>
export default {
  name: "button",
};
</script>
```

## Good

AppButton is an application component name and does not reuse the native button name.

```vue annotate="add:1,2,4,5,6,7,8,9"
<script setup lang="ts">
defineOptions({ name: "AppButton" });
</script>

<template>
  <Transition>
    <AppButton />
  </Transition>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-reserved-component-names": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L44)
