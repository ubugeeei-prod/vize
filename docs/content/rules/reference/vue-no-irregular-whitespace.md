---
title: "vue/no-irregular-whitespace"
description: "Disallow irregular whitespace in script blocks and templates."
---

# `vue/no-irregular-whitespace`

Disallow irregular whitespace in script blocks and templates.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The expression contains a non-breaking space.

```vue annotate="remove:2"
<template>
  <p>{{ value + other }}</p>
</template>
```

## Good

The expression uses ordinary source whitespace.

```vue annotate="add:2"
<template>
  <p>{{ value + other }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-irregular-whitespace": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Differences from eslint-plugin-vue

The script blocks and the template are scanned, including template text, comments, and attribute values. `<style>` is not scanned. A run of irregular spaces is one report. String literals are skipped. Comments, regular expressions, template literals, HTML attribute values, and HTML text are checked. The plugin options `skipStrings`, `skipComments`, `skipRegExps`, `skipTemplates`, `skipHTMLAttributeValues`, and `skipHTMLTextContents` are not configurable; the rule always uses those defaults, which skip only string literals.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_irregular_whitespace.rs#L20)
