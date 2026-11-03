---
title: "vize:croquis/cf/deep-import"
---

# `vize:croquis/cf/deep-import`

导入链超过项目允许的深度。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

这是显式选择的项目布局策略，并未宣称存在受支持的深度阈值或选项。此诊断约定目前没有实现。

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
import { label } from './entry';
</script>

<template>
<p>{{ label }}</p>
</template>

```

`value.ts`

```ts
export const label = 'Notice';

```

`level-one.ts`

```ts
export { label } from './level-two';

```

`level-two.ts`

```ts
export { label } from './level-three';

```

`level-three.ts`

```ts
export { label } from './value';

```

`public-api.ts`

```ts
export { label } from './value';

```

## 错误示例

入口让简单值经过 `level-one`、`level-two` 和 `level-three`，对期望浅层公共边界的项目形成不必要的深导入链。

`entry.ts`

```ts
export { label } from './level-one';

```

## 正确示例

入口使用直接重新导出该值的 `public-api.ts`。消费者保留相同导入名称，导入链变短。

`entry.ts`

```ts
export { label } from './public-api';

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](../cross-file.md)
