---
title: "a11y/mouse-events-have-key-events"
description: "Require focus/blur events with mouse events."
---

# `a11y/mouse-events-have-key-events`

Require focus/blur events with mouse events.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Preview visibility changes only through mouse enter and leave handlers.

```vue annotate="remove:2"
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

## Good

The same preview actions run on focus and blur, and the button can receive keyboard focus.

```vue annotate="add:2,3,4,5,6,7,8,9,10"
<template>
  <button
    type="button"
    @focus="showPreview"
    @blur="hidePreview"
    @mouseenter="showPreview"
    @mouseleave="hidePreview"
  >
    Preview
  </button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/mouse-events-have-key-events": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L32)
