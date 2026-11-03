---
title: "vize:croquis/cf/uncaught-error"
---

# `vize:croquis/cf/uncaught-error`

组件可能抛错，却没有错误边界捕获。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| info | 无；请检查相关文件并应用修复 | 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度 |

适用范围: 已分析的组件图，以及下方说明的受支持事实

当前诊断实现扫描 JSON.parse(input) 等模板表达式，不会报告仅存在于 script 块中的 throw 语句。

## 共享项目文件

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## 错误示例

子组件模板对格式错误的输入调用 `JSON.parse`，可达父组件没有错误捕获边界。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const input = "{";
</script>
<template><button @click="JSON.parse(input)">Parse</button></template>
```

## 正确示例

父组件为该子组件注册 `onErrorCaptured`。返回 `false` 会停止传播；生产环境边界还应提供有用的恢复界面。

`App.vue`

```vue
<script setup lang="ts">
import { onErrorCaptured } from "vue";
import Child from "./Child.vue";
onErrorCaptured(() => false);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const input = "{";
</script>
<template><button @click="JSON.parse(input)">Parse</button></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/uncaught-error": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 相关页面

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[跨文件规则索引](../cross-file.md)
