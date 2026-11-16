---
title: "vue/valid-attribute-name"
description: "Require valid attribute names."
---

# `vue/valid-attribute-name`

Require valid attribute names.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

Bad diagnostic: `parser/template`

Malformed attribute spelling is diagnosed by parser/template before this defensive rule sees an attribute. Bad therefore reports parser/template; it does not promise a separate vue/valid-attribute-name finding.

## Bad

The quote inside `my"attr` makes the attribute name malformed. This example produces the parser's `parser/template` diagnostic rather than promising a separate rule diagnostic.

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

## Good

`my-attr` is a well-formed attribute name, so the template parser can read the attribute and its value.

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-attribute-name": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L28)
