---
title: "html/cross-component-nesting"
---

# `html/cross-component-nesting`

Check actual HTML nesting after imported components are composed.

| Default severity | Automatic fix | Options |
| --- | --- | --- |
| warning | None | crossFile; rule severity (off/warn/error) |

Applies to: Reachable project declarations and imported components

## Shared project files

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## Bad

The parent's `<p>` contains a resolved child whose root is `<div>`, producing invalid paragraph/block nesting after composition.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><p><Child /></p></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

## Good

Use a `<section>` container that can contain the child's block element; the child stays unchanged.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><section><Child /></section></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "html/cross-component-nesting": "warn" },
  },
});
```

```sh
vp run lint
```

## Related

[Cross-file index](../cross-file.md)
