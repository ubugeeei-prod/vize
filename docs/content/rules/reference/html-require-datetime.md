---
title: "html/require-datetime"
description: "Require datetime attribute on <time> element."
---

# `html/require-datetime`

Require datetime attribute on &lt;time&gt; element.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The time element contains a human-readable date but no machine-readable datetime value.

```vue annotate="remove:2"
<template>
  <time>May 13, 2026</time>
</template>
```

## Good

`datetime="2026-05-13"` supplies the corresponding machine-readable date.

```vue annotate="add:2"
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [HTML rules](../html.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L36)
