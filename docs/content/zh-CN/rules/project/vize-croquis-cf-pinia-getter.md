---
title: "vize:croquis/cf/pinia-getter"
---

# `vize:croquis/cf/pinia-getter`

避免在 setup 中将 Pinia getter 的值复制为不再跟随更新的快照。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| 不产生诊断 | 无；请检查相关文件并应用修复 | 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度 |

适用范围: 已分析的组件图，以及下方说明的受支持事实

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

必须安装 Pinia，且 main.ts 须在挂载前安装其插件。在受跟踪的计算或模板内直接读取 `store.doubled` 是有效的；此处问题是取普通快照。该诊断约定目前没有实现。

## 适用范围与限制

这个约定展示了在 setup 中复制 Pinia getter 的当前原始类型值，得到不再跟随更新的快照。在受跟踪的计算或模板中直接读取 getter 仍可保持响应性；这里通过 storeToRefs() 提取值来修复问题。

## 共享项目文件

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
createApp(App).use(createPinia()).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`counter-store.ts`

```ts
import { defineStore } from 'pinia';
export const useCounterStore = defineStore('counter', {
  state: () => ({ count: 0 }),
  getters: { doubled: state => state.count * 2 },
});

```

## 错误示例

`const doubled = store.doubled` 在 setup 时复制 getter 的当前数字。复制的数字不会跟随后续 `store.count` 更新。

`App.vue`

```vue
<script setup lang="ts">
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const doubled = store.doubled;
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

## 正确示例

`storeToRefs(store)` 提供响应式 getter ref，可被解构并由模板解包，同时继续关联 store。

`App.vue`

```vue
<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const { doubled } = storeToRefs(store);
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

## 配置（Vite+）

CLI 当前不会输出这个诊断代码，因此这里没有可启用它的 CLI 规则配置。

## 相关页面

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](../cross-file.md)
