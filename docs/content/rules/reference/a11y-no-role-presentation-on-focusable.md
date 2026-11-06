---
title: "a11y/no-role-presentation-on-focusable"
description: "Disallow role=\"presentation\" or role=\"none\" on focusable elements."
---

# `a11y/no-role-presentation-on-focusable`

Disallow role="presentation" or role="none" on focusable elements.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The focusable billing link requests role=presentation, which conflicts with its interactive link role; browsers must ignore that presentation request.

```vue annotate="remove:2"
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

## Good

Remove the conflicting presentation request and rely on the native link role and billing destination.

```vue annotate="add:2"
<template>
  <a href="/billing">Billing</a>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-role-presentation-on-focusable": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19)
