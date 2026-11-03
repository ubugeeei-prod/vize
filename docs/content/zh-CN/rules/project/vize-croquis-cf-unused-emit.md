---
title: "vize:croquis/cf/unused-emit"
---

# `vize:croquis/cf/unused-emit`

已声明的 emit 从未使用。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| warning | 无；请检查相关文件并应用修复 | 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度 |

适用范围: 已分析的组件图，以及下方说明的受支持事实

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

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

子组件声明了 `save`，却从未以该名称调用事件发出函数。

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
const emit = defineEmits<{ save: [] }>();
</script>
<template><p>Content</p></template>
```

## 正确示例

示例调用 `emit("save")`，使声明事件得到使用。真实交互应在对应操作发生时发出它。

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
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

## 配置（Vite+）

CLI 当前不会输出这个诊断代码，因此这里没有可启用它的 CLI 规则配置。

## 相关页面

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[跨文件规则索引](../cross-file.md)
