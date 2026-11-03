---
title: "vue/cross-file-attrs-fallthrough"
---

# `vue/cross-file-attrs-fallthrough`

父组件传入属性，但已解析子组件的根无法继承它们，也没有显式使用 $attrs。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| warning | 无 | crossFile；规则严重程度（off/warn/error） |

适用范围: 可达的项目声明及导入组件

## 共享项目文件

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

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

## 错误示例

父组件将 `class="notice"` 传给已解析的片段子组件；该子组件没有自动属性目标，也从不读取 `$attrs`。

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

## 正确示例

子组件将 `$attrs` 绑定在 `<main>` 上，选择它作为目标；同级 `<aside>` 保持独立。

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

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "vue/cross-file-attrs-fallthrough": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 相关页面

[跨文件规则索引](../cross-file.md)
