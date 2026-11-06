---
title: "ecosystem/void-link-require-href"
description: "Require `href` on Void Vue Link components."
---

# `ecosystem/void-link-require-href`

Require `href` on Void Vue Link components.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `ecosystem` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The Link imported from @void/vue omits its href destination.

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

## Good

The same imported Link receives the settings destination through href.

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/void-link-require-href": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Ecosystem and Nuxt rules](../ecosystem.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13)
