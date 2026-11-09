---
title: "vue/no-content-outside-block"
description: "禁止 SFC 顶层块之外的内容。"
---

# `vue/no-content-outside-block`

禁止 SFC 顶层块之外的内容。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

开头文本位于所有 SFC 顶层块之外。

```vue annotate="remove:1,2"
unexpected
<template><div /></template>
```

## 正确示例

所有编写的内容都在 template 块内。

```vue annotate="add:1"
<template><div>expected</div></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-content-outside-block": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L40)
