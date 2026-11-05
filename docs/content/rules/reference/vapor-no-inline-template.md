---
title: "vapor/no-inline-template"
description: "Disallow deprecated inline-template attribute."
---

# `vapor/no-inline-template`

Disallow deprecated inline-template attribute.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

LegacyCard uses the inline-template attribute for its child markup.

```vue annotate="remove:2,3"
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

## Good

The markup is passed through the default slot instead of an inline template.

```vue annotate="add:2,3,4,5"
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
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
        "vapor/no-inline-template": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vapor rules](../vapor.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31)
