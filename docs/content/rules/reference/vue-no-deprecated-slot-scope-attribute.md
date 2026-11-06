---
title: "vue/no-deprecated-slot-scope-attribute"
description: "Disallow the deprecated `slot-scope` attribute."
---

# `vue/no-deprecated-slot-scope-attribute`

Disallow the deprecated `slot-scope` attribute.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The template receives slot props through the deprecated slot-scope attribute.

```vue annotate="remove:2"
<template>
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

## Good

The #default directive receives those props without slot-scope.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-slot-scope-attribute": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33)
