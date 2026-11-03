---
title: "vize:croquis/cf/closure-captures-reactive"
---

# `vize:croquis/cf/closure-captures-reactive`

闭包捕获了响应式值，无法看到后续更新。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

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

`App.vue`

```vue
<script setup lang="ts">
import { computed, ref } from 'vue';
import { makeReader } from './reader';
const count = ref(0);
const read = makeReader(count);
const shown = computed(read);
</script>

<template>
<button @click="count++">Increment {{ count }}</button><p>{{ shown }}</p>
</template>

```

## 错误示例

`makeReader` 在创建闭包前复制 `count.value`。计算读取器随后返回初始数字，没有读取响应式依赖。

`reader.ts`

```ts
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  const captured = count.value;
  return () => captured;
}

```

## 正确示例

闭包在调用时读取 `count.value`，使计算 getter 能跟踪 ref，并在递增后更新 `shown`。

`reader.ts`

```ts
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  return () => count.value;
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](../cross-file.md)
