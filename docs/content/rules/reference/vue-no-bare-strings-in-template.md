---
title: "vue/no-bare-strings-in-template"
description: "Disallow raw human-readable text in the template that should be internationalized."
---

# `vue/no-bare-strings-in-template`

Disallow raw human-readable text in the template that should be internationalized.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

The allowlist replaces eslint-plugin-vue's punctuation defaults when present. Entries are literal substrings or /pattern/flags values and are removed before the remaining text is checked. Omit allowlist to retain the defaults; [] clears them. Options do not enable the rule.

## Bad

Visible text and naming attributes embed untranslated strings directly in the template.

```vue annotate="remove:2"
<template>
<p>Translate me</p>
</template>
```

## Good

Translatable content calls $t; the punctuation and numeric-only examples are allowed exceptions.

```vue annotate="add:2"
<template>
<kbd>W</kbd>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-bare-strings-in-template": "warn"
    },
    "ruleOptions": {
      "vue/no-bare-strings-in-template": {
        "allowlist": [
          "/^[DRWXYmvx]$/u"
        ]
      }
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L51)
