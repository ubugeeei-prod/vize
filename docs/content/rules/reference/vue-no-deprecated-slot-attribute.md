---
title: "vue/no-deprecated-slot-attribute"
description: "Disallow the deprecated `slot` attribute."
---

# `vue/no-deprecated-slot-attribute`

Disallow the deprecated `slot` attribute.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The header slot is selected through the old slot attribute.

```vue annotate="remove:3,4"
<template>
  <Foo>
    <template slot="header"><h1>Title</h1></template>
    <div :slot="name">Title</div>
  </Foo>
</template>
```

## Good

v-slot:header explicitly selects the header slot with the current directive.

```vue annotate="add:3"
<template>
  <Foo>
    <template v-slot:header><h1>Title</h1></template>
  </Foo>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-slot-attribute": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L41)
