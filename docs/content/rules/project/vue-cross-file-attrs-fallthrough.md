---
title: "vue/cross-file-attrs-fallthrough"
---

# `vue/cross-file-attrs-fallthrough`

A parent passes attributes to a resolved child whose root cannot inherit them and does not explicitly use $attrs.

Default severity: warning  
Applies to: Reachable project declarations and imported components  
Options: crossFile; rule severity (off/warn/error)  
Automatic fix: None

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "vue/cross-file-attrs-fallthrough": "warn" },
    },
  },
});
```

```sh
vp run lint
```

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

The parent passes `class="notice"` to a resolved fragment child that has no automatic attribute target and never reads `$attrs`.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

## Good

The child chooses `<main>` as its target by binding `$attrs` there; its sibling `<aside>` remains separate.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Cross-file index](../cross-file.md)
