---
title: "vue/no-deprecated-v-on-native-modifier"
description: "Disallow the deprecated `.native` modifier on `v-on`."
---

# `vue/no-deprecated-v-on-native-modifier`

Disallow the deprecated `.native` modifier on `v-on`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The component handlers use the removed .native event modifier.

```vue annotate="remove:2,3,4"
<template>
<MyComponent @click.native="handler" />
<MyComponent v-on:click.native="handler" />
<MyComponent @click.native.stop="handler" />
</template>
```

## Good

The handlers omit .native and preserve other event modifiers such as .stop.

```vue annotate="add:2,3"
<template>
<MyComponent @click="handler" />
<MyComponent @click.stop="handler" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-v-on-native-modifier": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L45)
