---
title: "vize:croquis/cf/reference-escapes-scope"
---

# `vize:croquis/cf/reference-escapes-scope`

响应式引用逸出拥有其生命周期的作用域。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

Refs 可以合法地从组合式函数返回或跨作用域共享。此示例显式要求快照缓存，并未声称卸载会使 ref 失效。目前没有实现发出此诊断约定。

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

`saved.ts`

```ts
import type { Ref } from 'vue';
let saved: Ref<number> | number | undefined;
export function remember(value: Ref<number> | number): void { saved = value; }
export function remembered(): Ref<number> | number | undefined { return saved; }

```

## 错误示例

进程级缓存保留组件的活 count ref，可能让实例状态在卸载后仍可达，并观察后续编辑，但此缓存原本用于存储快照。

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { remember } from './saved';
const count = ref(0);
remember(count);
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

## 正确示例

缓存接收当前普通数字，保留快照而不保留组件拥有的 ref。

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { remember } from './saved';
const count = ref(0);
remember(count.value);
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](../cross-file.md)
