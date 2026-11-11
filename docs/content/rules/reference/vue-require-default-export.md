---
title: "vue/require-default-export"
description: "Require a component's script to be the default export."
---

# `vue/require-default-export`

Require a component's script to be the default export.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The component script has no default export.

```vue
<script lang="ts">
const label = "a";
</script>
<template>
  <p>label</p>
</template>
```

## Good

The component is the default export of the script.

```vue annotate="add:3"
<script lang="ts">
const label = "a";
export default { name: "Label" };
</script>
<template>
  <p>label</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-default-export": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_default_export.rs#L14)
