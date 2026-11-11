---
title: "a11y/role-img-has-name"
description: "Require an accessible name on elements with role=\"img\"."
---

# `a11y/role-img-has-name`

Require an accessible name on elements with role="img".

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The image sets `role="img"` after an empty `alt`, so it is exposed as an image with no accessible name.

```vue annotate="remove:2"
<template>
  <img src="/a.png" alt="" role="img" />
</template>
```

## Good

`alt="Network diagram"` names the image, so `role="img"` has an accessible name.

```vue annotate="add:2"
<template>
  <img src="/a.png" alt="Network diagram" role="img" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/role-img-has-name": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Differences from eslint-plugin-vuejs-accessibility

This rule reports an element whose explicit role is `img` when that element has no accessible name. `a11y/no-redundant-roles` keeps `role="img"` on `<img alt="">` because an empty `alt` makes the implicit role `presentation` under HTML-ARIA, so the explicit role is not redundant. eslint-plugin-vuejs-accessibility still reports that attribute as a redundant `img` role. WAI-ARIA 1.2 requires an accessible name for the `img` role. A non-empty `alt` on `<img>`, or `aria-label`, `aria-labelledby`, or `title`, supplies that name.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_img_has_name.rs#L14)
