---
title: "vue/a11y-img-alt"
description: "Require alt attribute on images for accessibility."
---

# `vue/a11y-img-alt`

Require alt attribute on images for accessibility.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Neither the static image nor the dynamically sourced image supplies an alt attribute.

```vue annotate="remove:2,3"
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

## Good

Informative images get descriptive alt text, decoration gets an empty alt, and the dynamic image binds its description.

```vue annotate="add:2,3,4,5,6,7,8,9"
<template>
<!-- Informative image -->
<img src="/photo.jpg" alt="Team photo from company retreat" />

<!-- Decorative image (empty alt) -->
<img src="/decoration.svg" alt="" />

<!-- Dynamic alt -->
<img :src="photo" :alt="photoDescription" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/a11y-img-alt": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33)
