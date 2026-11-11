---
title: "vue/no-unsupported-features"
description: "disallow unsupported Vue.js syntax on the specified version."
---

# `vue/no-unsupported-features`

disallow unsupported Vue.js syntax on the specified version.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

defineModel needs a newer Vue than the configured version.

```vue annotate="remove:2,5"
<script setup lang="ts">
const model = defineModel<string>();
</script>
<template>
  <p>{{ model }}</p>
</template>
```

## Good

The component uses no version-gated macro.

```vue annotate="add:2,5"
<script setup lang="ts">
const label = "ok";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unsupported-features": "warn"
    },
    "ruleOptions": {
      "vue/no-unsupported-features": {
        "version": "^3.3.0"
      }
    }
  },
});
```

Then run `vp run lint`.

## Differences from eslint-plugin-vue

The configured minimum is compared with the script macros `defineModel` (3.4.0), `defineOptions` (3.3.0), and `defineSlots` (3.3.0). Template syntax such as `v-memo`, slot shorthand, and dynamic directive arguments is outside this rule. An empty version checks nothing, and `ignores` skips a macro id such as `define-model`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/unsupported_features.rs#L19)
