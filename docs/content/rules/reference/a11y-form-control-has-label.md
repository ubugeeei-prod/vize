---
title: "a11y/form-control-has-label"
description: "Require form controls to have associated labels."
---

# `a11y/form-control-has-label`

Require form controls to have associated labels.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The search input has no label identifying what the user should enter.

```vue annotate="remove:2"
<template>
  <input type="search" />
</template>
```

## Good

Wrapping the input in a label associates the visible `Search` text with the control.

```vue annotate="add:2,3,4,5"
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/form-control-has-label": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Differences from eslint-plugin-vuejs-accessibility

These shapes stay different because they follow what Vue renders:

- `<input />` and `<input v-model="value" />` are reported. The default type is `text`, and `v-model` is not a label.
- `<input v-bind="attrs" />` is not reported. The spread can carry `id`, `aria-label`, or `aria-labelledby` at runtime.
- `<input hidden>` is not reported. The `hidden` attribute keeps the control out of the accessibility tree.
- JSX `title={title}` is not a label. In a Vue template, `:title="label"` is a label when the expression is not `null`, `undefined`, or `""`. `:title="undefined"` is still reported, because Vue omits it.

`<meter>`, `<progress>`, and `<output>` need a label. Text inside `<output>` is the value, not the label. Boolean `aria-hidden`, a binding other than `false`, and the same `aria-hidden` on an ancestor hide the control. `:aria-hidden="false"` still needs a label.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L21)
