---
title: "vize:croquis/cf/array-mutation"
---

# `vize:croquis/cf/array-mutation`

通过索引修改数组，无法被该响应式数组跟踪。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| 不产生诊断 | 无；请检查相关文件并应用修复 | 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度 |

适用范围: 已分析的组件图，以及下方说明的受支持事实

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

仅适用于历史 Vue 2.7：此情形须使用匹配的 Vue 2.7 和 SFC 编译器依赖。Vue 3 代理跟踪数组索引赋值，因此 `items[0] = next` 在 Vue 3 中具有响应性，并非 Vue 3 缺陷。此公开诊断代码目前没有实现。

## 共享项目文件

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import Vue from 'vue';
import App from './App.vue';
new Vue({ render: h => h(App) }).$mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script lang="ts">
import Vue from 'vue';
import { replaceFirst } from './replace-first';
export default Vue.extend({
  data() { return { items: ['Before'] }; },
  methods: { replace() { replaceFirst(this.items, 'After'); } },
});
</script>
<template><section><p>{{ items[0] }}</p><button @click="replace">Replace</button></section></template>

```

## 错误示例

在此历史 Vue 2.7 项目中，`items[0] = next` 修改数组却不通知 Vue 2 数组观察器，因此显示的首项不一定更新。

`replace-first.ts`

```ts
export function replaceFirst(items: string[], next: string): void {
  items[0] = next;
}

```

## 正确示例

`splice(0, 1, next)` 使用 Vue 2 可观察的数组修改方法，使同样的替换能更新视图。

`replace-first.ts`

```ts
export function replaceFirst(items: string[], next: string): void {
  items.splice(0, 1, next);
}

```

## 配置（Vite+）

CLI 当前不会输出这个诊断代码，因此这里没有可启用它的 CLI 规则配置。

## 相关页面

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](../cross-file.md)
