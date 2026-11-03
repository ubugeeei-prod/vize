---
title: "vize:croquis/cf/async-no-suspense"
---

# `vize:croquis/cf/async-no-suspense`

异步组件在没有 Suspense 边界的情况下渲染。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

当前支持情况: `no-source-async-fact`

边界诊断实现读取 macros.is_async()，但当前源代码解析将顶层 await 记录在 script-setup 作用域。因此，下方完整的错误与正确源代码对照不会通过当前 CLI 产生 async-no-suspense 诊断。它解释 Suspense 规范；补充缺失宏事实属于后续实现工作。

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

子组件具有顶层 await，但父组件未提供 `<Suspense>` 边界。当前源代码解析未提供发出此诊断代码所需的宏事实。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

## 正确示例

父组件将同一异步子组件包在 `<Suspense>` 中，并提供加载后备内容。这展示规范；当前分析遍对两种源代码形式都不发出该诊断。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Suspense><Child /><template #fallback><p>Loading</p></template></Suspense></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[跨文件规则索引](../cross-file.md)
