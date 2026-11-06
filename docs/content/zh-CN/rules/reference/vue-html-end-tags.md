---
title: "vue/html-end-tags"
description: "要求显式书写 HTML 中可以省略的结束标签。"
---

# `vue/html-end-tags`

要求显式书写 HTML 中可以省略的结束标签。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

本规则要求显式书写 HTML 解析器允许省略的结束标签，例如 `li`、`p` 和 `td`。缺失的必需结束标签由 `parser/template` 诊断；空元素不需要结束标签。

## 错误示例

HTML 允许在 `ul` 结束时隐式关闭这个 `li`，但本规则要求显式书写结束标签。

```vue annotate="remove:1"
<template><ul><li>First</ul></template>
```

## 正确示例

在列表项内容之后、外层 `ul` 结束之前加上 `</li>`。

```vue annotate="add:1"
<template><ul><li>First</li></ul></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/html-end-tags": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_end_tags.rs#L30)
