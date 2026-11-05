---
title: "vize:croquis/cf/reactivity-outside-setup"
---

# `vize:croquis/cf/reactivity-outside-setup`

响应式 API 在 `setup` 外被调用。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| 不产生诊断 | 无；请检查相关文件并应用修复 | 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度 |

适用范围: 已分析的组件图，以及下方说明的受支持事实

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

Vue 允许在组件 setup 外使用 ref/reactive/computed。此处风险是在显式实例隔离策略下产生非预期所有权与共享，并非 API 不合法。此诊断约定目前没有实现。

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
import { useCounter } from './use-counter';
const { count, doubled } = useCounter();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>
</template>

```

## 错误示例

两个响应式 API 都在模块加载时运行，因此两个 Counter 实例共享一个 ref 和计算值，违背独立计数器的预期。

`use-counter.ts`

```ts
import { computed, ref } from 'vue';
const count = ref(0);
const doubled = computed(() => count.value * 2);
export function useCounter() { return { count, doubled }; }

```

## 正确示例

`useCounter` 在每次组件 setup 调用中同步创建 ref 和计算值，为每个控件提供自己的状态及受跟踪的派生值。

`use-counter.ts`

```ts
import { computed, ref } from 'vue';
export function useCounter() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

## 配置（Vite+）

CLI 当前不会输出这个诊断代码，因此这里没有可启用它的 CLI 规则配置。

## 相关页面

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](../cross-file.md)
