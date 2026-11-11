---
title: "a11y/interactive-supports-focus"
description: "Require interactive role elements to be focusable."
---

# `a11y/interactive-supports-focus`

Require interactive role elements to be focusable.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

Unlike eslint-plugin-vuejs-accessibility, a listbox that owns a focusable option (roving tabindex) or that a combobox controls through `aria-activedescendant` needs no tab stop of its own, a bound `tabindex` whose conditional branches are all integers (`:tabindex="active ? 0 : -1"`) counts as a tab index, and `:aria-hidden="false"` does not hide the element.

## Bad

Giving a `span` the button role and a click handler does not make the element keyboard-focusable.

```vue annotate="remove:2"
<template>
  <span role="button" @click="open">Open</span>
</template>
```

## Good

The native button is focusable and retains the same `open` action.

```vue annotate="add:2"
<template>
  <button type="button" @click="open">Open</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/interactive-supports-focus": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L46)
