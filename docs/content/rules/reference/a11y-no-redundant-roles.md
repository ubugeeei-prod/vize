---
title: "a11y/no-redundant-roles"
description: "Disallow redundant ARIA roles."
---

# `a11y/no-redundant-roles`

Disallow redundant ARIA roles.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The native button already has the button role, so `role="button"` repeats its implicit semantics.

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

## Good

Removing the repeated role keeps the button semantics supplied by HTML.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-redundant-roles": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Differences from eslint-plugin-vuejs-accessibility

These shapes follow the HTML-ARIA mapping, or an existing static-role boundary:

- `<img alt="" role="presentation">` is reported. An empty `alt` is `presentation`.
- `<img alt="" role="img">` is not reported. `role="img"` overrides that presentation role.
- `<summary role="button">` is reported. `summary` already has the `button` role.
- `<select multiple role="listbox">` and `<select size="3" role="listbox">` are reported. `multiple`, or a static `size` greater than 1, is a listbox.
- `<a :href="href" role="link">` is not reported. A dynamic `href` is not a static link. A static `href`, including `href=""`, is.
- A bound role such as `<button :role="'button'">` is not reported. Only a static `role` attribute is compared, and the comparison is exact.
- `<ul>` and `<ol>` stay lists, including a list whose markers are removed with `list-style: none`.

An unnamed `<nav>`, `<section>`, or `<form>` has no corresponding landmark role, so `role="navigation"`, `role="region"`, or `role="form"` on that element is not redundant. A name is `aria-label`, `aria-labelledby`, or `title`. A default `<select>` is a combobox. `div` and `span` are `generic`. `p` is `paragraph`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31)
