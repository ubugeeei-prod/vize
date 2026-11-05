---
title: "vize:croquis/cf/module-scope-reactive"
---

# `vize:croquis/cf/module-scope-reactive`

响应式状态在模块作用域创建，被所有调用者共享。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| 不产生诊断 | 无；请检查相关文件并应用修复 | 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度 |

适用范围: 已分析的组件图，以及下方说明的受支持事实

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

有意共享的应用 store 在模块作用域创建响应式状态是合法的。此示例假定组件与请求隔离；此公开诊断约定目前没有实现。

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
import Counter from './Counter.vue';
</script>

<template>
<Counter /><Counter />
</template>

```

`Counter.vue`

```vue
<script setup lang="ts">
import { createCounter } from './counter';
const { count } = createCounter();
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

## 错误示例

模块仅初始化一次 `count`，两个 Counter 实例收到同一个 ref。虽然示例期望独立实例状态，点击一个却会改变两个计数器。

`counter.ts`

```ts
import { ref } from 'vue';
const count = ref(0);
export function createCounter() { return { count }; }

```

## 正确示例

在 `createCounter` 内创建 ref，为每个同步 setup 调用提供独立状态对象，使每个按钮拥有自己的计数器。

`counter.ts`

```ts
import { ref } from 'vue';
export function createCounter() {
  const count = ref(0);
  return { count };
}

```

## 配置（Vite+）

CLI 当前不会输出这个诊断代码，因此这里没有可启用它的 CLI 规则配置。

## 相关页面

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](../cross-file.md)
