---
title: "ecosystem/vue-router-missing-param"
---

# `ecosystem/vue-router-missing-param`

Required postId is missing; relying on the current route is fragile.

Default severity: warning  
Applies to: Reachable project declarations and imported components  
Options: crossFile; rule severity (off/warn/error)  
Automatic fix: None

The complete installed router must be reachable from the application's createApp(...).use(router). Unknown/dynamic route tables do not prove unknown-name findings. Missing params are warnings because navigation may inherit a value from the current route.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-missing-param": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## Shared project files

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

## Bad

The navigation omits required `postId` from the `user-post` path. This is a warning because Vue Router may inherit a value from the current route.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1" } });
</script>
<template><p>Post</p></template>
```

## Good

Pass both `userId` and `postId` explicitly so navigation does not depend on the current route's parameter state.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Cross-file index](../cross-file.md)
