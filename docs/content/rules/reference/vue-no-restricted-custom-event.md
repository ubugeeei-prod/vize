---
title: "vue/no-restricted-custom-event"
description: "disallow specific custom event."
---

# `vue/no-restricted-custom-event`

disallow specific custom event.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The template emits the forbidden legacyEvent.

```vue annotate="remove:2"
<template>
  <button type="button" @click="$emit('legacyEvent')">x</button>
</template>
```

## Good

The template emits save.

```vue annotate="add:2"
<template>
  <button type="button" @click="$emit('save')">x</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-custom-event": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-custom-event": [
        "legacyEvent"
      ]
    }
  },
});
```

Then run `vp run lint`.

## Differences from eslint-plugin-vue

Template `$emit` calls are reported. A call through a `setup()` parameter or `this.$emit` stays quiet, and so does the binding `defineEmits` returns.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_custom_event.rs#L19)
