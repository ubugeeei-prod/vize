---
title: "vue/no-textarea-mustache"
description: "Disallow mustache interpolation in `<textarea>`."
---

# `vue/no-textarea-mustache`

Disallow mustache interpolation in `<textarea>`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The textarea places message in child interpolation instead of binding its value.

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

## Good

v-model binds the editable textarea value to message.

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-textarea-mustache": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26)
