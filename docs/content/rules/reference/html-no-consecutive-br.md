---
title: "html/no-consecutive-br"
description: "Disallow consecutive <br> elements."
---

# `html/no-consecutive-br`

Disallow consecutive &lt;br&gt; elements.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Two consecutive break elements create spacing between blocks inside a single paragraph.

```vue annotate="remove:2"
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

## Good

Separate paragraphs express the two content blocks without repeated break elements.

```vue annotate="add:2,3"
<template>
  <p>First line</p>
  <p>Second block</p>
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
        "html/no-consecutive-br": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [HTML rules](../html.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30)
