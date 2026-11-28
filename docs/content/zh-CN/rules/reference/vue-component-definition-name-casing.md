---
title: "vue/component-definition-name-casing"
description: "要求组件定义名称使用 PascalCase 或 kebab-case。"
---

# `vue/component-definition-name-casing`

要求组件定义名称使用 PascalCase 或 kebab-case。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

检查组件定义的静态名称：name 选项、defineOptions({ name }) 和 app.component() 的第一个参数。文件名由 vue/component-file-name-casing 检查。

## 错误示例

通过 defineOptions 声明的名称 my_component 用下划线分隔单词，不是 PascalCase。

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "my_component" });
</script>
<template><p>Content</p></template>
```

## 正确示例

声明的名称 MyComponent 使用默认的 PascalCase。

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "MyComponent" });
</script>
<template><p>Content</p></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/component-definition-name-casing": "warn"
    },
    "ruleOptions": {
      "vue/component-definition-name-casing": "PascalCase"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L68)
