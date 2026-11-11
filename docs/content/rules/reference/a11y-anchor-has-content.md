---
title: "a11y/anchor-has-content"
description: "Require anchor elements to have accessible content."
---

# `a11y/anchor-has-content`

Require anchor elements to have accessible content.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

List component names whose rendered subtree always supplies accessible anchor content. PascalCase and kebab-case spellings are equivalent. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.

## Bad

The `/app` link has no text or other naming content, so its destination has no accessible description.

```vue annotate="remove:2"
<template>
<a href="/app"></a>
</template>
```

## Good

The native image's `alt` text supplies accessible content for the same destination link.

```vue annotate="add:2"
<template>
<a href="/app"><img src="/badge.png" alt="Get the app" /></a>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/anchor-has-content": "warn"
    },
    "ruleOptions": {
      "a11y/anchor-has-content": {
        "accessibleChildren": [
          "RetryImage"
        ]
      }
    }
  },
});
```

Then run `vp run lint`.

## Differences from eslint-plugin-vuejs-accessibility

These shapes stay different because they follow what Vue renders, or because a descendant accessible name is already content:

- `<a><svg role="img" aria-label="Settings">…</svg></a>` is not reported. The nested `aria-label` names the link.
- `<a><MyIcon aria-label="Settings"></MyIcon></a>` is not reported for the same reason.
- `<a :aria-label="undefined">` is reported. Vue omits a binding whose value is `undefined`.

A `<slot>` is a content hole, so `aria-hidden` on the slot itself does not hide the slotted nodes. `v-text` and `v-html` on the anchor count when the expression is not `null`, `undefined`, or `""`. `{{ undefined }}` is empty. Boolean `aria-hidden`, and a binding other than `false`, hides that element. An anchor that is itself hidden needs no content. `:aria-hidden="false"` still does.

## Related

- [Accessibility rules](../accessibility.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L24)
