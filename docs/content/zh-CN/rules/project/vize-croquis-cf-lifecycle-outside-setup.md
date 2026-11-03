---
title: "vize:croquis/cf/lifecycle-outside-setup"
---

# `vize:croquis/cf/lifecycle-outside-setup`

生命周期钩子在 `setup` 外注册。

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

`install-title.ts`

```ts
import { onMounted } from 'vue';
export function installTitle() {
  onMounted(() => { document.title = 'Mounted application'; });
}

```

## 错误示例

入口在挂载应用前调用 `installTitle()`，因此 `onMounted` 在没有激活组件 setup 上下文时注册。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
import { installTitle } from './install-title';
installTitle();
createApp(App).mount('#app');

```

`App.vue`

```vue
<script setup lang="ts">

</script>

<template>
<p>Application</p>
</template>

```

## 正确示例

从 App 的 setup 同步调用同一辅助函数，将生命周期回调关联到该实例的挂载。

`App.vue`

```vue
<script setup lang="ts">
import { installTitle } from './install-title';
installTitle();
</script>

<template>
<p>Application</p>
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](../cross-file.md)
