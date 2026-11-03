---
title: "vize:croquis/cf/event-listener-leak"
---

# `vize:croquis/cf/event-listener-leak`

注册了事件监听器，却从未移除。

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
import { useWidth } from './use-width';
const width = useWidth();
</script>

<template>
<p>{{ width }}</p>
</template>

```

## 错误示例

挂载时添加的 window resize 监听器捕获组件 width ref，但卸载时从不移除。重复挂载可能保留未使用的监听器和状态。

`use-width.ts`

```ts
import { onMounted, ref } from 'vue';
export function useWidth() {
  const width = ref(0);
  const resize = () => { width.value = window.innerWidth; };
  onMounted(() => { resize(); window.addEventListener('resize', resize); });
  return width;
}

```

## 正确示例

`onUnmounted` 移除挂载时注册的同一个 `resize` 函数，结束该实例外部监听器的生命周期。

`use-width.ts`

```ts
import { onMounted, onUnmounted, ref } from 'vue';
export function useWidth() {
  const width = ref(0);
  const resize = () => { width.value = window.innerWidth; };
  onMounted(() => { resize(); window.addEventListener('resize', resize); });
  onUnmounted(() => { window.removeEventListener('resize', resize); });
  return width;
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](../cross-file.md)
