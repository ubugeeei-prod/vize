---
title: "vize:croquis/cf/undefined-slot"
---

# `vize:croquis/cf/undefined-slot`

父组件填充了子组件未暴露的插槽。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| 不产生诊断 | 无；请检查相关文件并应用修复 | 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度 |

适用范围: 已分析的组件图，以及下方说明的受支持事实

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

## 共享项目文件

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`Card.vue`

```vue
<script setup lang="ts">
defineSlots<{ header(): unknown }>();
</script>

<template>
<article><header><slot name="header" /></header></article>
</template>

```

## 错误示例

App 提供 `footer` 插槽，但 Card 仅声明和渲染 `header`，传入的 Notice 内容在此子组件中没有匹配的插槽出口。

`App.vue`

```vue
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #footer>Notice</template></Card>
</template>

```

## 正确示例

App 提供 `header`，与子组件类型化插槽声明和渲染出口一致，因此 Notice 在该处显示。

`App.vue`

```vue
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #header>Notice</template></Card>
</template>

```

## 配置（Vite+）

CLI 当前不会输出这个诊断代码，因此这里没有可启用它的 CLI 规则配置。

## 相关页面

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](../cross-file.md)
