---
title: "vue/html-button-has-type"
description: "Require an explicit valid type on button elements."
---

# `vue/html-button-has-type`

Require an explicit valid type on button elements.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

One button omits type and another supplies the unsupported foo type.

```vue annotate="remove:2,3"
<template>
<button>Click</button>
<button type="foo">Click</button>
</template>
```

## Good

Buttons specify button, submit, or reset; a bound type is treated as dynamic.

```vue annotate="add:2,3,4,5"
<template>
<button type="button">Click</button>
<button type="submit">Save</button>
<button type="reset">Reset</button>
<button :type="dynamicType">Click</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/html-button-has-type": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39)
