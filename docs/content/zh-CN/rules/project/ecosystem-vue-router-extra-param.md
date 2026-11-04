---
title: "ecosystem/vue-router-extra-param"
---

# `ecosystem/vue-router-extra-param`

报告目标路由未声明的导航参数。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| error | 无 | crossFile；规则严重程度（off/warn/error） |

适用范围: 可达的项目声明及导入组件

完整且已安装的路由器必须从应用的 createApp(...).use(router) 可达。未知或动态路由表不能证明未知名称诊断。缺失参数属于警告，因为导航可能从当前路由继承值。

## 共享项目文件

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

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

## 错误示例

路由未声明 tab；Vue Router 会丢弃它。`user-post` 路径声明 `userId` 和 `postId`，但导航还将未声明的 `tab` 作为路径参数传入。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2", tab: "a" } });
</script>
<template><p>Post</p></template>
```

## 正确示例

从 params 移除 `tab`，只保留路由路径中的键。应用需要选择标签页时，应另行使用 query。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-extra-param": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 相关页面

[跨文件规则索引](../cross-file.md)
