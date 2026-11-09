---
title: "vue/no-define-model-default"
description: "禁止 defineModel 中的默认值。"
---

# `vue/no-define-model-default`

禁止 defineModel 中的默认值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

子 model 声明了可能与父级不同步的默认值。

```vue annotate="remove:2"
<script setup>
const model = defineModel({ default: "draft" })
</script>
<template />
```

## 正确示例

初始 model 值由父级提供。

```vue annotate="add:2"
<script setup>
const model = defineModel<string>()
</script>
<template />
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-define-model-default": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L44)
