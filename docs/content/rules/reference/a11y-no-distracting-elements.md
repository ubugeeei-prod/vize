---
title: "a11y/no-distracting-elements"
description: "Disallow distracting elements like <marquee> and <blink>."
---

# `a11y/no-distracting-elements`

Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt;.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The `marquee` element introduces automatically moving text.

```vue annotate="remove:2"
<template>
  <marquee>Limited offer</marquee>
</template>
```

## Good

A paragraph displays the same offer without the distracting marquee element.

```vue annotate="add:2"
<template>
  <p>Limited offer</p>
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
        "a11y/no-distracting-elements": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16)
