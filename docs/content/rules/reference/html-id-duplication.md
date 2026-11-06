---
title: "html/id-duplication"
description: "Disallow duplicate element IDs."
---

# `html/id-duplication`

Disallow duplicate element IDs.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Both the input and help paragraph declare `id="email"`, so the label target is ambiguous.

```vue annotate="remove:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

## Good

The input keeps `email`; the help paragraph uses `email-help`, and aria-describedby refers to that distinct ID.

```vue annotate="add:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/id-duplication": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [HTML rules](../html.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36)
