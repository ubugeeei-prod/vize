---
title: "vue/max-template-depth"
description: "enforce maximum depth of template."
---

# `vue/max-template-depth`

enforce maximum depth of template.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The span is nested deeper than the configured maximum.

```vue annotate="remove:2,3,4,5,6,7,8"
<template>
  <div>
    <section>
      <div>
        <span>deep</span>
      </div>
    </section>
  </div>
</template>
```

## Good

The template has a single element.

```vue annotate="add:2"
<template>
  <p>ok</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/max-template-depth": "warn"
    },
    "ruleOptions": {
      "vue/max-template-depth": {
        "maxDepth": 3
      }
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/max_template_depth.rs#L13)
