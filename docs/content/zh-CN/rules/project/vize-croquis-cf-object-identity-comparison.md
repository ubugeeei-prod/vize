---
title: "vize:croquis/cf/object-identity-comparison"
---

# `vize:croquis/cf/object-identity-comparison`

用稳定的记录 ID 比较原始对象与响应式代理所表示的记录。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| 不产生诊断 | 无；请检查相关文件并应用修复 | 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度 |

适用范围: 已分析的组件图，以及下方说明的受支持事实

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

此示例假定 ID 唯一标识记录。比较指向同一响应式代理的两个引用仍然有效；此诊断约定目前没有实现。原始对象与其响应式代理具有不同的对象身份，toRaw() 返回原始对象。

## 适用范围与限制

原始对象与其响应式代理具有不同的对象身份，toRaw() 返回原始对象。这个约定展示了在需要判断记录身份时，比较稳定的记录 ID 的方法。

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

`user.ts`

```ts
import { reactive } from 'vue';
export function makeUser() {
  const raw = { id: 7, name: 'Ada' };
  return { raw, proxy: reactive(raw) };
}

```

## 错误示例

此约定示例在意图判断记录身份时比较稳定的记录 ID。`proxy === raw` 比较包装对象身份，因此即使二者代表同一用户记录，结果也是 false。应用希望比较记录身份，而非包装对象身份。

`App.vue`

```vue
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy === raw;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

## 正确示例

比较稳定的记录 `id`，回答预期问题，不依赖对象是原始对象还是代理。

`App.vue`

```vue
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy.id === raw.id;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

## 配置（Vite+）

CLI 当前不会输出这个诊断代码，因此这里没有可启用它的 CLI 规则配置。

## 相关页面

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](../cross-file.md)
