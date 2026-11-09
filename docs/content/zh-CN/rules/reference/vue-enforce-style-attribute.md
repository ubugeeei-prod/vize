---
title: "vue/enforce-style-attribute"
description: "将 style 块限制为允许的属性模式。"
---

# `vue/enforce-style-attribute`

将 style 块限制为允许的属性模式。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

module 样式模式不在配置的允许列表中。

```vue annotate="remove:2"
<template><div /></template>
<style module>.card { color: red }</style>
```

## 正确示例

scoped 样式模式符合项目策略。

```vue annotate="add:2"
<template><div /></template>
<style scoped>.card { color: red }</style>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/enforce-style-attribute": "warn"
    },
    "ruleOptions": {
      "vue/enforce-style-attribute": {
        "vue/enforce-style-attribute": {
          "allow": [
            "scoped"
          ]
        }
      }
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L35)
