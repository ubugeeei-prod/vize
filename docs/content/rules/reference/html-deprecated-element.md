---
title: "html/deprecated-element"
description: "Disallow deprecated HTML elements."
---

# `html/deprecated-element`

Disallow deprecated HTML elements.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The `center` element uses a deprecated HTML presentation element.

```vue annotate="remove:2"
<template>
  <center>Profile</center>
</template>
```

## Good

A section and a styling class replace the deprecated element while preserving the content.

```vue annotate="add:2"
<template>
  <section class="profile">Profile</section>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/deprecated-element": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [HTML rules](../html.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33)
