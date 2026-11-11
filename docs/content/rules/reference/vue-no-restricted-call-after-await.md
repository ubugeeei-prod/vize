---
title: "vue/no-restricted-call-after-await"
description: "disallow asynchronously called restricted methods."
---

# `vue/no-restricted-call-after-await`

disallow asynchronously called restricted methods.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

setup calls onMounted after await.

```vue annotate="remove:4,5"
<script lang="ts">
import { defineComponent, onMounted } from "vue";
export default defineComponent({
  async setup() {
    await Promise.resolve();
    onMounted(() => {});
  },
});
</script>
<template>
  <p>ok</p>
</template>
```

## Good

setup registers onMounted without await.

```vue annotate="add:4"
<script lang="ts">
import { defineComponent, onMounted } from "vue";
export default defineComponent({
  setup() {
    onMounted(() => {});
  },
});
</script>
<template>
  <p>ok</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-call-after-await": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-call-after-await": [
        {
          "module": "vue",
          "path": "onMounted"
        }
      ]
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/await_call.rs#L25)
