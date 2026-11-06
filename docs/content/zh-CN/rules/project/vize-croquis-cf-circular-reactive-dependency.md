---
title: "vize:croquis/cf/circular-reactive-dependency"
---

# `vize:croquis/cf/circular-reactive-dependency`

响应式计算相互依赖，形成循环。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| 依上下文而定 | 无；请检查相关文件并应用修复 | 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度 |

适用范围: 已分析的组件图，以及下方说明的受支持事实

示例验证状态: `illustrative-source-pair`

下方完整 Vue 项目展示更新反馈及其修复，但不构成已验证的 CLI 诊断复现：诊断实现需要附图所示的保留响应式流引用身份和边。这些源代码不能证明当前源代码路径会发出此确切代码。专门的跟踪 ID 图诊断对照仍与源代码语法检查分开。

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
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`count-key.ts`

```ts
import type { InjectionKey, Ref } from 'vue';
export const countKey: InjectionKey<Ref<number>> = Symbol('count');
```

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from 'vue';
import { countKey } from './count-key';
import CycleView from './CycleView.vue';
const count = ref(1); // A: the provider-owned source.
provide(countKey, count);
</script>
<template>
  <button @click="count++">Increment</button>
  <CycleView />
</template>
```

## 错误示例

App 拥有并提供 count（A）。CycleView 派生 nextCount（B），随后立即将每个派生值写回同一注入的 count。每次写入再次改变计算输入，形成 A → B → A 更新反馈。下方保留图中的身份代表这两个引用，而非仅名称相同的无关绑定。

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject, watch } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
watch(nextCount, value => { count.value = value; }, { immediate: true });
</script>
<template><p>{{ nextCount }}</p></template>
```

```diagram
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B; B -> A
```

## 正确示例

移除将 B 写回 A 的侦听器。App 保留 count 的所有权，仅通过显式 Increment 操作改变它；CycleView 读取派生 nextCount，不将结果反馈回去。同样的引用只保留 A → B 依赖。

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
</script>
<template><p>{{ nextCount }}</p></template>
```

```diagram
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "croquis/cf/circular-reactive-dependency": "warn" },
  },
});
```

```sh
vp run lint
```

## 相关页面

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[跨文件规则索引](../cross-file.md)
