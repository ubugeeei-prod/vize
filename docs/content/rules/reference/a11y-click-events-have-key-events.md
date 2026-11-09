---
title: "a11y/click-events-have-key-events"
description: "Require keyboard event handlers with click events."
---

# `a11y/click-events-have-key-events`

Require keyboard event handlers with click events.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

Checks non-interactive elements without an interactive role. Native buttons and elements with an interactive ARIA role are outside this rule's finding.

## Bad

The non-interactive `div` has a click handler but no keyboard event handling.

```vue annotate="remove:2"
<template>
<div @click="activate">Activate</div>
</template>
```

## Good

A native `button` provides keyboard activation for the same `activate` handler.

```vue annotate="add:2"
<template>
<button @click="activate">Activate</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/click-events-have-key-events": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L25)
