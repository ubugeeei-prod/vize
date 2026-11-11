---
title: "vue/prop-name-casing"
description: "规范所声明 prop 名称的大小写风格。"
---

# `vue/prop-name-casing`

规范所声明 prop 名称的大小写风格。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

检查所声明的 prop 名称，而不是传给子组件的属性的大小写风格。

## 错误示例

声明的 prop 名称 user_name 使用下划线分隔。

```vue annotate="remove:2,4"
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

## 正确示例

声明及其模板引用都使用驼峰名称 userName。

```vue annotate="add:2,4"
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prop-name-casing": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L52)
