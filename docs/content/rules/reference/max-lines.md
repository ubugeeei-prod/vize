---
title: "max-lines"
description: "Enforce a maximum number of lines in a file."
---

# `max-lines`

Enforce a maximum number of lines in a file.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The file has more lines than the configured maximum.

```vue annotate="remove:4,6,7,8,9,10,11,12,13,14,15,16,17"
<script setup lang="ts">
const label = "a";
</script>

<template>
  <ul>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
  </ul>
</template>

```

## Good

The file stays within the configured maximum.

```vue annotate="add:5"
<script setup lang="ts">
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>

```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "max-lines": "warn"
    },
    "ruleOptions": {
      "max-lines": {
        "max": 10
      }
    }
  },
});
```

Then run `vp run lint`.

## Differences from ESLint

The count covers the whole file Vize lints, including `<template>` and `<style>`, so an SFC can be capped in one place. A trailing newline does not add a line. `skipComments` skips a line whose remaining text is only a `//` comment, a `/* */` comment, or an HTML comment, and it does not treat text inside a string as a comment. A comment that shares a line with code stays counted. ESLint takes those comment lines from the parser's comment list.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/max_lines.rs#L15)
