---
title: "vue/html-self-closing"
description: "Enforce self-closing style."
---

# `vue/html-self-closing`

Enforce self-closing style.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Yes | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The empty component and normal div use closing pairs instead of the configured self-closing spelling.

```vue annotate="remove:3,4"
<template>
  <section>
    <MyComponent></MyComponent>
    <div></div>
  </section>
</template>
```

## Good

Empty components and normal elements use self-closing syntax, while void elements omit the slash and an element with content retains its closing tag.

```vue annotate="add:3,4,5,6,7"
<template>
  <section>
    <MyComponent />
    <div />
    <img>
    <br>
    <div>content</div>
  </section>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/html-self-closing": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L31)
