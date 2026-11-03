---
title: "vize:croquis/cf/provide-inject-type"
---

# `vize:croquis/cf/provide-inject-type`

提供值与其 inject 类型不一致。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| warning | 无；请检查相关文件并应用修复 | 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度 |

适用范围: 已分析的组件图，以及下方说明的受支持事实

此检查比较提供者与消费者的显式类型注解，而非推断的字面量值类型。保留示例中提供者的 `as string` 注解。

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

提供者将 `title` 显式注解为 `string`，后代却为同一键请求 `inject<number>`。

`App.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import Child from "./Child.vue";
provide("title", "Hello" as string);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
const title = inject<number>("title");
</script>
<template><p>Content</p></template>
```

## 正确示例

消费者显式的 `inject<string>` 与提供者注解一致。保留 `as string`：此诊断产生逻辑比较显式注解，而非推断的字面量类型。

`App.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import Child from "./Child.vue";
provide("title", "Hello" as string);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
const title = inject<string>("title");
</script>
<template><p>Content</p></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/provide-inject-type": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 相关页面

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis/diagnostics.rs)

[跨文件规则索引](../cross-file.md)
