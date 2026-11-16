---
title: "vue/attribute-hyphenation"
description: "Enforce attribute naming style on custom components."
---

# `vue/attribute-hyphenation`

Enforce attribute naming style on custom components.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The component attribute uses the camelCase spelling firstName.

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

## Good

The first-name spelling follows the configured hyphenated component-attribute convention.

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/attribute-hyphenation": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35)
