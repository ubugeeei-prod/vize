---
title: "vize:croquis/cf/circular-dep"
---

# `vize:croquis/cf/circular-dep`

组件相互导入，形成循环。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

此示例展示具体的立即初始化循环。递归 Vue 组件或任意循环导入并不自动构成错误。目前没有实现发出此诊断约定代码。

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

`App.vue`

```vue
<script setup lang="ts">
import { aLabel } from './a';
</script>

<template>
<p>{{ aLabel }}</p>
</template>

```

`labels.ts`

```ts
export const aPrefix = 'A';
export const bPrefix = 'B';

```

## 错误示例

`a.ts` 导入 `b.ts`，后者又导入 `a.ts`。两者都立即读取另一模块尚未初始化的常量来初始化自身常量，导致暂时性死区错误。

`a.ts`

```ts
import { bLabel } from './b';
export const aLabel = 'A' + bLabel;

```

`b.ts`

```ts
import { aLabel } from './a';
export const bLabel = 'B' + aLabel;

```

## 正确示例

两个模块都从独立的 `labels.ts` 读取已初始化前缀，移除循环及立即发生的交叉读取。

`a.ts`

```ts
import { bPrefix } from './labels';
export const aLabel = 'A' + bPrefix;

```

`b.ts`

```ts
import { aPrefix } from './labels';
export const bLabel = 'B' + aPrefix;

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](../cross-file.md)
