---
title: "html/cross-component-nesting"
---

# `html/cross-component-nesting`

检查导入组件组合后的实际 HTML 嵌套。

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

父组件的 `<p>` 包含根为 `<div>` 的已解析子组件，组合后形成无效的段落与块级元素嵌套。

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

## 正确示例

改用可容纳子组件块级元素的 `<section>` 容器；子组件保持不变。

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

## 配置（Vite+）

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

## 相关页面

[跨文件规则索引](../cross-file.md)
